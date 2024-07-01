/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::collections::VecDeque;
use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use kernel_guard::NoPreemptIrqSave;
use lazy_init::LazyInit;
use ruxfdtable::{FD_TABLE, RUX_FILE_LIMIT};
use scheduler::BaseScheduler;
use spinlock::{BaseSpinLock, Combine, ExpRand, SpinNoIrq};

use crate::task::{CurrentTask, TaskState};
use crate::{AxTaskRef, Scheduler, TaskInner, WaitQueue};

pub(crate) const BACKOFF_LIMIT: u32 = 8;
pub(crate) type DefaultStrategy = Combine<ExpRand<BACKOFF_LIMIT>, spinlock::NoOp>;
pub(crate) type RQLock<T> = BaseSpinLock<NoPreemptIrqSave, T, DefaultStrategy>;

// TODO: per-CPU
pub(crate) static RUN_QUEUE: LazyInit<RQLock<AxRunQueue>> = LazyInit::new();

// TODO: per-CPU
static EXITED_TASKS: SpinNoIrq<VecDeque<AxTaskRef>> = SpinNoIrq::new(VecDeque::new());

static WAIT_FOR_EXIT: WaitQueue = WaitQueue::new();

#[percpu::def_percpu]
static IDLE_TASK: LazyInit<AxTaskRef> = LazyInit::new();

pub(crate) struct AxRunQueue {
    scheduler: Scheduler,
}

impl AxRunQueue {
    pub fn new() -> RQLock<Self> {
        let gc_task = TaskInner::new(gc_entry, "gc".into(), ruxconfig::TASK_STACK_SIZE);
        let mut scheduler = Scheduler::new();
        scheduler.add_task(gc_task);
        RQLock::new(Self { scheduler })
    }

    pub fn add_task(&mut self, task: AxTaskRef) {
        debug!("task spawn: {}", task.id_name());
        assert!(task.is_ready());
        self.scheduler.add_task(task);
    }

    #[cfg(feature = "irq")]
    pub fn scheduler_timer_tick(&mut self) {
        use crate::loadavg;
        let curr = crate::current();
        loadavg::calc_load_tick(curr.is_idle());
        if !curr.is_idle() && self.scheduler.task_tick(curr.as_task_ref()) {
            #[cfg(feature = "preempt")]
            curr.set_preempt_pending(true);
        }
    }

    pub fn yield_current(&mut self) {
        let curr = crate::current();
        trace!("task yield: {}", curr.id_name());
        assert!(curr.is_running());
        self.resched(false);
    }

    pub fn set_current_priority(&mut self, prio: isize) -> bool {
        self.scheduler
            .set_priority(crate::current().as_task_ref(), prio)
    }

    #[cfg(feature = "preempt")]
    pub fn preempt_resched(&mut self) {
        let curr = crate::current();
        assert!(curr.is_running());

        // When we get the mutable reference of the run queue, we must
        // have held the `SpinNoIrq` lock with both IRQs and preemption
        // disabled. So we need to set `current_disable_count` to 1 in
        // `can_preempt()` to obtain the preemption permission before
        //  locking the run queue.
        let can_preempt = curr.can_preempt(1);

        debug!(
            "current task is to be preempted: {}, allow={}",
            curr.id_name(),
            can_preempt
        );
        if can_preempt {
            self.resched(true);
        } else {
            curr.set_preempt_pending(true);
        }
    }

    pub fn exit_current(&mut self, exit_code: i32) -> ! {
        let curr = crate::current();
        debug!("task exit: {}, exit_code={}", curr.id_name(), exit_code);
        assert!(curr.is_running());
        assert!(!curr.is_idle());
        if curr.is_init() {
            EXITED_TASKS.lock().clear();
            ruxhal::misc::terminate();
        } else {
            curr.set_state(TaskState::Exited);
            curr.notify_exit(exit_code, self);
            EXITED_TASKS.lock().push_back(curr.clone());
            WAIT_FOR_EXIT.notify_one_locked(false, self);
            self.resched(false);
        }
        unreachable!("task exited!");
    }

    pub fn block_current<F>(&mut self, wait_queue_push: F)
    where
        F: FnOnce(AxTaskRef),
    {
        let curr = crate::current();
        debug!("task block: {}", curr.id_name());
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        // we must not block current task with preemption disabled.
        #[cfg(feature = "preempt")]
        assert!(curr.can_preempt(1));

        curr.set_state(TaskState::Blocked);
        wait_queue_push(curr.clone());
        self.resched(false);
    }

    pub fn unblock_task(&mut self, task: AxTaskRef, resched: bool) {
        debug!("task unblock: {}", task.id_name());
        if task.is_blocked() {
            task.set_state(TaskState::Ready);
            self.scheduler.add_task(task); // TODO: priority
            if resched {
                #[cfg(feature = "preempt")]
                crate::current().set_preempt_pending(true);
            }
        }
    }

    #[cfg(feature = "irq")]
    pub fn sleep_until(&mut self, deadline: ruxhal::time::TimeValue) {
        let curr = crate::current();
        debug!("task sleep: {}, deadline={:?}", curr.id_name(), deadline);
        assert!(curr.is_running());
        assert!(!curr.is_idle());

        let now = ruxhal::time::current_time();
        if now < deadline {
            crate::timers::set_alarm_wakeup(deadline, curr.clone());
            curr.set_state(TaskState::Blocked);
            self.resched(false);
        }
    }
}

impl AxRunQueue {
    /// Common reschedule subroutine. If `preempt`, keep current task's time
    /// slice, otherwise reset it.
    fn resched(&mut self, preempt: bool) {
        let prev = crate::current();
        if prev.is_running() {
            prev.set_state(TaskState::Ready);
            if !prev.is_idle() {
                self.scheduler.put_prev_task(prev.clone(), preempt);
            }
        }
        let next = self.scheduler.pick_next_task().unwrap_or_else(|| unsafe {
            // Safety: IRQs must be disabled at this time.
            IDLE_TASK.current_ref_raw().get_unchecked().clone()
        });
        self.switch_to(prev, next);
    }

    fn switch_to(&mut self, prev_task: CurrentTask, next_task: AxTaskRef) {
        trace!(
            "context switch: {} -> {}",
            prev_task.id_name(),
            next_task.id_name()
        );
        #[cfg(feature = "preempt")]
        next_task.set_preempt_pending(false);
        next_task.set_state(TaskState::Running);
        if prev_task.ptr_eq(&next_task) {
            return;
        }

        unsafe {
            let prev_ctx_ptr = prev_task.ctx_mut_ptr();
            let next_ctx_ptr = next_task.ctx_mut_ptr();

            // The strong reference count of `prev_task` will be decremented by 1,
            // but won't be dropped until `gc_entry()` is called.
            assert!(Arc::strong_count(prev_task.as_task_ref()) > 1);
            assert!(Arc::strong_count(&next_task) >= 1);

            CurrentTask::set_current(prev_task, next_task);
            (*prev_ctx_ptr).switch_to(&*next_ctx_ptr);
        }
    }
}

fn gc_flush_file(fd: usize) -> LinuxResult {
    trace!("gc task flush: {}", fd);
    FD_TABLE
        .read()
        .get(fd)
        .cloned()
        .ok_or(LinuxError::EBADF)?
        .flush()
}

fn gc_entry() {
    let mut now_file_fd: usize = 3;
    loop {
        let _ = gc_flush_file(now_file_fd);
        now_file_fd += 1;
        if now_file_fd >= RUX_FILE_LIMIT {
            now_file_fd = 3;
        }
        // Drop all exited tasks and recycle resources.
        let n = EXITED_TASKS.lock().len();
        for _ in 0..n {
            // Do not do the slow drops in the critical section.
            let task = EXITED_TASKS.lock().pop_front();
            if let Some(task) = task {
                if Arc::strong_count(&task) == 1 {
                    // If I'm the last holder of the task, drop it immediately.
                    drop(task);
                } else {
                    // Otherwise (e.g, `switch_to` is not compeleted, held by the
                    // joiner, etc), push it back and wait for them to drop first.
                    EXITED_TASKS.lock().push_back(task);
                }
            }
        }
        WAIT_FOR_EXIT.wait();
    }
}

pub(crate) fn init() {
    const IDLE_TASK_STACK_SIZE: usize = 4096;
    let idle_task = TaskInner::new(|| crate::run_idle(), "idle".into(), IDLE_TASK_STACK_SIZE);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));

    let main_task = TaskInner::new_init("main".into());
    main_task.set_state(TaskState::Running);

    RUN_QUEUE.init_by(AxRunQueue::new());
    unsafe { CurrentTask::init_current(main_task) }
}

pub(crate) fn init_secondary() {
    let idle_task = TaskInner::new_init("idle".into());
    idle_task.set_state(TaskState::Running);
    IDLE_TASK.with_current(|i| i.init_by(idle_task.clone()));
    unsafe { CurrentTask::init_current(idle_task) }
}
