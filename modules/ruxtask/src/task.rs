/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::fs::FileSystem;
use alloc::collections::BTreeMap;
use alloc::{
    boxed::Box,
    string::String,
    sync::{Arc, Weak},
};
use core::ops::Deref;
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicU64, AtomicU8, Ordering};
use core::{alloc::Layout, cell::UnsafeCell, fmt, ptr::NonNull};
use page_table::PageSize;
use page_table_entry::MappingFlags;
use ruxhal::mem::direct_virt_to_phys;
#[cfg(feature = "paging")]
use ruxhal::{mem::phys_to_virt, paging::PageTable};
use spinlock::SpinNoIrq;

#[cfg(feature = "preempt")]
use core::sync::atomic::AtomicUsize;

#[cfg(feature = "tls")]
use ruxhal::tls::TlsArea;

use memory_addr::{align_up_4k, VirtAddr, PAGE_SIZE_4K};
use ruxhal::arch::{flush_tlb, TaskContext};

#[cfg(not(feature = "musl"))]
use crate::tsd::{DestrFunction, KEYS, TSD};
use crate::vma::MmapStruct;
use crate::current;
use crate::{AxRunQueue, AxTask, AxTaskRef, WaitQueue};

/// A unique identifier for a thread.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct TaskId(u64);

/// The possible states of a task.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum TaskState {
    /// Task is running on cpu
    Running = 1,
    /// Task is ready for schedule
    Ready = 2,
    /// Task is blocked
    Blocked = 3,
    /// Task exits, waiting for gc
    Exited = 4,
}

/// The inner task structure.
pub struct TaskInner {
    parent_process: Option<Weak<AxTask>>,
    process_task: Weak<AxTask>,
    id: TaskId,
    name: String,
    is_idle: bool,
    is_init: bool,
    entry: Option<*mut dyn FnOnce()>,
    state: AtomicU8,

    in_wait_queue: AtomicBool,
    #[cfg(feature = "irq")]
    in_timer_list: AtomicBool,

    #[cfg(feature = "preempt")]
    need_resched: AtomicBool,
    #[cfg(feature = "preempt")]
    preempt_disable_count: AtomicUsize,

    exit_code: AtomicI32,
    wait_for_exit: WaitQueue,

    kstack: SpinNoIrq<Arc<Option<TaskStack>>>,
    ctx: UnsafeCell<TaskContext>,

    #[cfg(feature = "tls")]
    tls: TlsArea,

    #[cfg(not(feature = "musl"))]
    tsd: TSD,

    // set tid
    #[cfg(feature = "musl")]
    set_tid: AtomicU64,
    // clear tid
    #[cfg(feature = "musl")]
    tl: AtomicU64,
    #[cfg(feature = "paging")]
    // The page table of the task.
    pub pagetable: Arc<SpinNoIrq<PageTable>>,
    // file system
    pub fs: Arc<SpinNoIrq<Option<FileSystem>>>,
    // memory management
    pub mm: Arc<MmapStruct>,
}

impl TaskId {
    fn new() -> Self {
        static ID_COUNTER: AtomicU64 = AtomicU64::new(1);
        Self(ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }

    /// Convert the task ID to a `u64`.
    pub const fn as_u64(&self) -> u64 {
        self.0
    }
}

impl From<u8> for TaskState {
    #[inline]
    fn from(state: u8) -> Self {
        match state {
            1 => Self::Running,
            2 => Self::Ready,
            3 => Self::Blocked,
            4 => Self::Exited,
            _ => unreachable!(),
        }
    }
}

unsafe impl Send for TaskInner {}
unsafe impl Sync for TaskInner {}

impl TaskInner {
    /// Gets the ID of the task.
    pub const fn id(&self) -> TaskId {
        self.id
    }

    /// Gets the clear tid of the task.
    #[cfg(feature = "musl")]
    pub const fn tl(&self) -> &AtomicU64 {
        &self.tl
    }

    /// Gets the name of the task.
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Get a combined string of the task ID and name.
    pub fn id_name(&self) -> alloc::string::String {
        alloc::format!("Task({}, {:?})", self.id.as_u64(), self.name)
    }

    /// Get pointer for parent process task
    pub fn parent_process(&self) -> Option<AxTaskRef> {
        if let Some(parent_process) = self.parent_process.as_ref() {
            return parent_process.upgrade();
        }
        None
    }

    /// Get process task
    pub fn process_task(&self) -> Arc<AxTask> {
        if let Some(process_task) = self.process_task.upgrade() {
            process_task.clone()
        } else {
            current().as_task_ref().clone()
        }
    }

    /// Get pid of the process of the task.
    pub fn process_id(&self) -> TaskId {
        if let Some(process_task) = self.process_task.upgrade() {
            process_task.id
        } else {
            self.id
        }
    }

    /// Wait for the task to exit, and return the exit code.
    ///
    /// It will return immediately if the task has already exited (but not dropped).
    pub fn join(&self) -> Option<i32> {
        self.wait_for_exit
            .wait_until(|| self.state() == TaskState::Exited);
        Some(self.exit_code.load(Ordering::Acquire))
    }

    /// set 0 to thread_list_lock
    #[cfg(feature = "musl")]
    pub fn free_thread_list_lock(&self) {
        let addr = self.tl.load(Ordering::Relaxed);
        if addr == 0 {
            return;
        }
        unsafe { &*(addr as *const AtomicI32) }.store(0, Ordering::Release)
    }
}

static PROCESS_MAP: SpinNoIrq<BTreeMap<u64, Arc<AxTask>>> = SpinNoIrq::new(BTreeMap::new());

use log::error;
// private methods
impl TaskInner {
    // clone a thread
    fn new_common(id: TaskId, name: String) -> Self {
        error!(
            "new_common: process_id={:#}, name={:?}",
            current().id_name(),
            id.0
        );
        Self {
            parent_process: Some(Arc::downgrade(current().as_task_ref())),
            process_task: Arc::downgrade(&current().process_task()),
            id,
            name,
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: SpinNoIrq::new(Arc::new(None)),
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(),
            #[cfg(not(feature = "musl"))]
            tsd: spinlock::SpinNoIrq::new([core::ptr::null_mut(); ruxconfig::PTHREAD_KEY_MAX]),
            #[cfg(feature = "musl")]
            set_tid: AtomicU64::new(0),
            #[cfg(feature = "musl")]
            tl: AtomicU64::new(0),
            #[cfg(feature = "paging")]
            pagetable: current().pagetable.clone(),
            fs: current().fs.clone(),
            mm: current().mm.clone(),
        }
    }

    #[cfg(feature = "musl")]
    fn new_common_tls(
        id: TaskId,
        name: String,
        #[cfg_attr(not(feature = "tls"), allow(unused_variables))] tls: usize,
        set_tid: AtomicU64,
        tl: AtomicU64,
    ) -> Self {
        use crate::current;
        Self {
            parent_process: Some(Arc::downgrade(current().as_task_ref())),
            process_task: Arc::downgrade(&current().process_task()),
            id,
            name,
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: SpinNoIrq::new(Arc::new(None)),
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::new_with_addr(tls),
            set_tid,
            // clear child tid
            tl,
            #[cfg(feature = "paging")]
            pagetable: current().pagetable.clone(),
            fs: current().fs.clone(),
            mm: current().mm.clone(),
        }
    }

    pub fn stack_top(&self) -> VirtAddr {
        self.kstack.lock().as_ref().as_ref().unwrap().top()
    }

    pub fn set_stack_top(&self, begin: usize, size: usize) {
        error!("set_stack_top: begin={:#x}, size={:#x}", begin, size);
        *self.kstack.lock() = Arc::new(Some(TaskStack {
            ptr: NonNull::new(begin as *mut u8).unwrap(),
            layout: Layout::from_size_align(size, PAGE_SIZE_4K).unwrap(),
        }));
    }

    /// for set_tid_addr
    #[cfg(feature = "musl")]
    pub fn set_child_tid(&self, tid: usize) {
        self.set_tid.store(tid as _, Ordering::Release);
    }

    /// Create a new task with the given entry function, stack size and tls area address
    #[cfg(feature = "musl")]
    pub(crate) fn new_musl<F>(
        entry: F,
        name: String,
        stack_size: usize,
        tls: usize,
        set_tid: AtomicU64,
        // clear child tid
        tl: AtomicU64,
    ) -> AxTaskRef
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common_tls(TaskId::new(), name, tls, set_tid, tl);
        let kstack = TaskStack::alloc(align_up_4k(stack_size));
        #[cfg(feature = "tls")]
        let tls = VirtAddr::from(t.tls.tls_ptr() as usize);
        #[cfg(not(feature = "tls"))]
        let tls = VirtAddr::from(0);

        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top(), tls);
        t.kstack = SpinNoIrq::new(Arc::new(Some(kstack)));
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    /// Create a new task with the given entry function and stack size.
    pub(crate) fn new<F>(entry: F, name: String, stack_size: usize) -> AxTaskRef
    where
        F: FnOnce() + Send + 'static,
    {
        let mut t = Self::new_common(TaskId::new(), name);
        debug!("new task: {}", t.id_name());
        let kstack = TaskStack::alloc(align_up_4k(stack_size));

        #[cfg(feature = "tls")]
        let tls = VirtAddr::from(t.tls.tls_ptr() as usize);
        #[cfg(not(feature = "tls"))]
        let tls = VirtAddr::from(0);

        t.entry = Some(Box::into_raw(Box::new(entry)));
        t.ctx.get_mut().init(task_entry as usize, kstack.top(), tls);
        t.kstack = SpinNoIrq::new(Arc::new(Some(kstack)));
        if t.name == "idle" {
            t.is_idle = true;
        }
        Arc::new(AxTask::new(t))
    }

    pub fn fork() -> AxTaskRef {
        use crate::alloc::string::ToString;

        let current_task = crate::current();
        let name = current_task.as_task_ref().name().to_string();
        let current_stack_bindings = current_task.as_task_ref().kstack.lock();
        let current_stack = current_stack_bindings.as_ref().as_ref().clone().unwrap();
        let current_stack_top = current_stack.top();
        let stack_size = current_stack.layout.size();
        debug!(
            "fork: current_stack_top={:#x}, stack_size={:#x}",
            current_stack_top, stack_size
        );

        #[cfg(feature = "paging")]
        // TODO: clone parent page table, and mark all unshared pages to read-only
        let mut cloned_page_table = PageTable::try_new().expect("failed to create page table");
        let cloned_mm = current().mm.as_ref().clone();
        
        // clone the global shared pages (as system memory)
        // TODO: exclude the stack page from the cloned page table
        #[cfg(feature = "paging")]
        for r in ruxhal::mem::memory_regions() {
            cloned_page_table
                .map_region(
                    phys_to_virt(r.paddr),
                    r.paddr,
                    r.size,
                    r.flags.into(),
                    false,
                )
                .expect("failed to map region when forking");
        }

        // mapping the page for stack to the process's stack, stack must keep at the same position.
        // TODO: merge these code with previous.
        #[cfg(feature = "paging")]
        let new_stack = TaskStack::alloc(align_up_4k(stack_size));
        let new_stack_vaddr = new_stack.end();
        let stack_paddr = direct_virt_to_phys(new_stack_vaddr);

        // Note: the stack region is mapped to the same position as the parent process's stack, be careful when update the stack region for the forked process.
        let (_, prev_flag, _) = cloned_page_table
            .query(current_stack.end())
            .expect("failed to query stack region when forking");
        cloned_page_table
            .unmap_region(current_stack.end(), align_up_4k(stack_size))
            .expect("failed to unmap stack region when forking");
        cloned_page_table
            .map_region(
                current_stack.end(),
                stack_paddr,
                stack_size,
                prev_flag,
                true,
            )
            .expect("failed to map stack region when forking");

        // clone parent pages in memory, and mark all unshared pages to read-only
        for (vaddr, page_info) in cloned_mm.mem_map.lock().iter() {
            let paddr = page_info.paddr;
            cloned_page_table
                .map((*vaddr).into(), paddr, PageSize::Size4K, MappingFlags::READ)
                .expect("failed to map when forking");
        }

        // mark the parent process's page table to read-only.
        for (vaddr, _) in current_task.mm.mem_map.lock().iter() {
            let mut page_table = current_task.pagetable.lock();
            let vaddr = VirtAddr::from(*vaddr);
            let (_, mapping_flag, _) = page_table
                .query(vaddr)
                .expect("Inconsistent page table with mem_map");
            if mapping_flag.contains(MappingFlags::EXECUTE) {
                page_table
                    .update(
                        vaddr,
                        None,
                        Some(MappingFlags::READ | MappingFlags::EXECUTE),
                    )
                    .expect("failed to update mapping when forking");

                cloned_page_table
                    .update(
                        vaddr,
                        None,
                        Some(MappingFlags::READ | MappingFlags::EXECUTE),
                    )
                    .expect("failed to update mapping when forking");
            } else {
                page_table
                    .update(vaddr, None, Some(MappingFlags::READ))
                    .expect("failed to update mapping when forking");
                
            }
            flush_tlb(Some(vaddr));
        }

        let mut t = Self {
            parent_process: Some(Arc::downgrade(current_task.as_task_ref())),
            process_task: Weak::new(),
            id: TaskId::new(),
            name,
            is_idle: false,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: SpinNoIrq::new(Arc::new(Some(new_stack))),
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(),
            #[cfg(not(feature = "musl"))]
            tsd: spinlock::SpinNoIrq::new([core::ptr::null_mut(); ruxconfig::PTHREAD_KEY_MAX]),
            #[cfg(feature = "musl")]
            set_tid: AtomicU64::new(0),
            #[cfg(feature = "musl")]
            tl: AtomicU64::new(0),
            #[cfg(feature = "paging")]
            pagetable: Arc::new(SpinNoIrq::new(cloned_page_table)),
            fs: Arc::new(SpinNoIrq::new(current_task.fs.lock().clone())),
            mm: Arc::new(cloned_mm),
        };

        debug!("new task forked: {}", t.id_name());

        #[cfg(feature = "tls")]
        let tls = VirtAddr::from(t.tls.tls_ptr() as usize);
        #[cfg(not(feature = "tls"))]
        let tls = VirtAddr::from(0);

        t.entry = None;
        t.ctx.get_mut().init(
            task_entry as usize,
            t.kstack.lock().as_ref().as_ref().unwrap().top(),
            tls,
        );
        let task_ref = Arc::new(AxTask::new(t));

        warn!(
            "start: copy stack content: current_stack_top={:#x} => new_stack_addr={:#x}",
            current_stack.end(),
            new_stack_vaddr
        );
        unsafe {
            // copy the stack content from current stack to new stack
            (*task_ref.ctx_mut_ptr()).save_current_content(
                current_stack.end().as_ptr(),
                new_stack_vaddr.as_mut_ptr(),
                stack_size,
            );
        }
        warn!(
            "end: copy stack content: current_stack_top={:#x} => new_stack_addr={:#x}",
            current_stack.end(),
            new_stack_vaddr
        );

        task_ref
    }

    /// Creates an "init task" using the current CPU states, to use as the
    /// current task.
    ///
    /// As it is the current task, no other task can switch to it until it
    /// switches out.
    ///
    /// And there is no need to set the `entry`, `kstack` or `tls` fields, as
    /// they will be filled automatically when the task is switches out.
    pub(crate) fn new_init(name: String) -> AxTaskRef {
        let mut t = Self {
            parent_process: None,
            process_task: Weak::new(),
            id: TaskId::new(),
            name,
            is_idle: false,
            is_init: true,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: SpinNoIrq::new(Arc::new(None)),
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(),
            #[cfg(not(feature = "musl"))]
            tsd: spinlock::SpinNoIrq::new([core::ptr::null_mut(); ruxconfig::PTHREAD_KEY_MAX]),
            #[cfg(feature = "musl")]
            set_tid: AtomicU64::new(0),
            #[cfg(feature = "musl")]
            tl: AtomicU64::new(0),
            #[cfg(feature = "paging")]
            pagetable: Arc::new(SpinNoIrq::new(
                PageTable::try_new().expect("failed to create page table"),
            )),
            fs: Arc::new(SpinNoIrq::new(None)),
            mm: Arc::new(MmapStruct::new()),
        };
        error!("new init task: {}", t.id_name());
        t.set_stack_top(boot_stack as usize, ruxconfig::TASK_STACK_SIZE);
        t.ctx.get_mut().init(
            task_entry as usize,
            VirtAddr::from(boot_stack as usize),
            VirtAddr::from(t.tls.tls_ptr() as usize),
        );
        let task_ref = Arc::new(AxTask::new(t));
        PROCESS_MAP
            .lock()
            .insert(task_ref.id().as_u64(), task_ref.clone());
        task_ref
    }

    pub fn new_idle(name: String) -> AxTaskRef {
        let bindings = PROCESS_MAP.lock();
        let (&_parent_id, &ref task_ref) = bindings.first_key_value().unwrap();
        let t = Self {
            parent_process: Some(Arc::downgrade(task_ref)),
            process_task: task_ref.process_task.clone(),
            id: TaskId::new(),
            name,
            is_idle: true,
            is_init: false,
            entry: None,
            state: AtomicU8::new(TaskState::Ready as u8),
            in_wait_queue: AtomicBool::new(false),
            #[cfg(feature = "irq")]
            in_timer_list: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            need_resched: AtomicBool::new(false),
            #[cfg(feature = "preempt")]
            preempt_disable_count: AtomicUsize::new(0),
            exit_code: AtomicI32::new(0),
            wait_for_exit: WaitQueue::new(),
            kstack: SpinNoIrq::new(Arc::new(None)),
            ctx: UnsafeCell::new(TaskContext::new()),
            #[cfg(feature = "tls")]
            tls: TlsArea::alloc(),
            #[cfg(not(feature = "musl"))]
            tsd: spinlock::SpinNoIrq::new([core::ptr::null_mut(); ruxconfig::PTHREAD_KEY_MAX]),
            #[cfg(feature = "musl")]
            set_tid: AtomicU64::new(0),
            #[cfg(feature = "musl")]
            tl: AtomicU64::new(0),
            #[cfg(feature = "paging")]
            pagetable: task_ref.pagetable.clone(),
            fs: task_ref.fs.clone(),
            mm: task_ref.mm.clone(),
        };

        Arc::new(AxTask::new(t))
    }

    /// Get task state
    #[inline]
    pub fn state(&self) -> TaskState {
        self.state.load(Ordering::Acquire).into()
    }

    /// Set task state
    #[inline]
    pub fn set_state(&self, state: TaskState) {
        self.state.store(state as u8, Ordering::Release)
    }

    #[inline]
    pub(crate) fn is_running(&self) -> bool {
        matches!(self.state(), TaskState::Running)
    }

    #[inline]
    pub(crate) fn is_ready(&self) -> bool {
        matches!(self.state(), TaskState::Ready)
    }

    /// Check blocking
    #[inline]
    pub fn is_blocked(&self) -> bool {
        matches!(self.state(), TaskState::Blocked)
    }

    #[inline]
    pub(crate) const fn is_init(&self) -> bool {
        self.is_init
    }

    #[inline]
    pub(crate) const fn is_idle(&self) -> bool {
        self.is_idle
    }

    #[inline]
    pub(crate) fn in_wait_queue(&self) -> bool {
        self.in_wait_queue.load(Ordering::Acquire)
    }

    #[inline]
    pub(crate) fn set_in_wait_queue(&self, in_wait_queue: bool) {
        self.in_wait_queue.store(in_wait_queue, Ordering::Release);
    }

    #[inline]
    #[cfg(feature = "irq")]
    pub(crate) fn in_timer_list(&self) -> bool {
        self.in_timer_list.load(Ordering::Acquire)
    }

    #[inline]
    #[cfg(feature = "irq")]
    pub(crate) fn set_in_timer_list(&self, in_timer_list: bool) {
        self.in_timer_list.store(in_timer_list, Ordering::Release);
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn set_preempt_pending(&self, pending: bool) {
        self.need_resched.store(pending, Ordering::Release)
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn can_preempt(&self, current_disable_count: usize) -> bool {
        self.preempt_disable_count.load(Ordering::Acquire) == current_disable_count
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn disable_preempt(&self) {
        self.preempt_disable_count.fetch_add(1, Ordering::Relaxed);
    }

    #[inline]
    #[cfg(feature = "preempt")]
    pub(crate) fn enable_preempt(&self, resched: bool) {
        if self.preempt_disable_count.fetch_sub(1, Ordering::Relaxed) == 1 && resched {
            // If current task is pending to be preempted, do rescheduling.
            Self::current_check_preempt_pending();
        }
    }

    #[cfg(feature = "preempt")]
    fn current_check_preempt_pending() {
        let curr = crate::current();
        if curr.need_resched.load(Ordering::Acquire) && curr.can_preempt(0) {
            let mut rq = crate::RUN_QUEUE.lock();
            if curr.need_resched.load(Ordering::Acquire) {
                rq.preempt_resched();
            }
        }
    }

    pub(crate) fn notify_exit(&self, exit_code: i32, rq: &mut AxRunQueue) {
        self.exit_code.store(exit_code, Ordering::Release);
        self.wait_for_exit.notify_all_locked(false, rq);
    }

    #[inline]
    pub(crate) const unsafe fn ctx_mut_ptr(&self) -> *mut TaskContext {
        self.ctx.get()
    }
}

#[cfg(not(feature = "musl"))]
impl TaskInner {
    /// Allocate a key
    pub fn alloc_key(&self, destr_function: Option<DestrFunction>) -> Option<usize> {
        unsafe { KEYS.lock() }.alloc(destr_function)
    }
    /// Get the destructor function of a key
    pub fn free_key(&self, key: usize) -> Option<()> {
        unsafe { KEYS.lock() }.free(key)
    }
    /// Get the destructor function of a key
    pub fn set_tsd(&self, key: usize, value: *mut core::ffi::c_void) -> Option<()> {
        if key < self.tsd.lock().len() {
            self.tsd.lock()[key] = value;
            Some(())
        } else {
            None
        }
    }
    /// Get the destructor function of a key
    pub fn get_tsd(&self, key: usize) -> Option<*mut core::ffi::c_void> {
        if key < self.tsd.lock().len() {
            Some(self.tsd.lock()[key])
        } else {
            None
        }
    }
    /// Get the destructor function of a key
    pub fn destroy_keys(&self) {
        unsafe { KEYS.lock() }.destr_used_keys(&self.tsd)
    }
}

impl fmt::Debug for TaskInner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TaskInner")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("state", &self.state())
            .finish()
    }
}

impl Drop for TaskInner {
    fn drop(&mut self) {
        error!("task drop: {}", self.id_name());
    }
}

#[derive(Debug)]
pub struct TaskStack {
    ptr: NonNull<u8>,
    layout: Layout,
}

impl TaskStack {
    pub fn alloc(size: usize) -> Self {
        let layout = Layout::from_size_align(size, 8).unwrap();
        debug!("taskStack::layout = {:?}", layout);
        Self {
            ptr: NonNull::new(unsafe { alloc::alloc::alloc(layout) }).unwrap(),
            layout,
        }
    }

    pub const fn top(&self) -> VirtAddr {
        unsafe { core::mem::transmute(self.ptr.as_ptr().add(self.layout.size())) }
    }

    pub const fn end(&self) -> VirtAddr {
        unsafe { core::mem::transmute(self.ptr.as_ptr()) }
    }

    pub fn size(&self) -> usize {
        self.layout.size()
    }
}

impl Drop for TaskStack {
    fn drop(&mut self) {
        warn!(
            "taskStack drop: ptr={:#x}, size={:#x}",
            self.ptr.as_ptr() as usize,
            self.layout.size()
        );
        unsafe { alloc::alloc::dealloc(self.ptr.as_ptr(), self.layout) }
    }
}

use core::mem::ManuallyDrop;

/// A wrapper of [`AxTaskRef`] as the current task.
pub struct CurrentTask(ManuallyDrop<AxTaskRef>);

impl CurrentTask {
    pub(crate) fn try_get() -> Option<Self> {
        let ptr: *const super::AxTask = ruxhal::cpu::current_task_ptr();
        if !ptr.is_null() {
            Some(Self(unsafe { ManuallyDrop::new(AxTaskRef::from_raw(ptr)) }))
        } else {
            None
        }
    }

    pub(crate) fn get() -> Self {
        Self::try_get().expect("current task is uninitialized")
    }

    /// Converts [`CurrentTask`] to [`AxTaskRef`].
    pub fn as_task_ref(&self) -> &AxTaskRef {
        &self.0
    }

    pub fn clone(&self) -> AxTaskRef {
        self.0.deref().clone()
    }

    pub(crate) fn ptr_eq(&self, other: &AxTaskRef) -> bool {
        Arc::ptr_eq(&self.0, other)
    }

    pub(crate) unsafe fn init_current(init_task: AxTaskRef) {
        #[cfg(feature = "tls")]
        ruxhal::arch::write_thread_pointer(init_task.tls.tls_ptr() as usize);
        let ptr = Arc::into_raw(init_task);
        ruxhal::cpu::set_current_task_ptr(ptr);
    }

    pub(crate) unsafe fn set_current(prev: Self, next: AxTaskRef) {
        error!(
            "-----------set_current-------------,next ptr={:#}",
            next.id_name()
        );
        let Self(arc) = prev;
        ManuallyDrop::into_inner(arc); // `call Arc::drop()` to decrease prev task reference count.
        let ptr = Arc::into_raw(next);
        ruxhal::cpu::set_current_task_ptr(ptr);
    }
}

impl Deref for CurrentTask {
    type Target = TaskInner;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

extern "C" fn task_entry() -> ! {
    // release the lock that was implicitly held across the reschedule
    unsafe { crate::RUN_QUEUE.force_unlock() };
    #[cfg(feature = "irq")]
    ruxhal::arch::enable_irqs();
    let task = crate::current();
    if let Some(entry) = task.entry {
        unsafe {
            let in_entry = Box::from_raw(entry);
            in_entry()
        };
    }
    crate::exit(0);
}

extern "C" {
    fn boot_stack();
}
