/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Task APIs for multi-task configuration.
use alloc::{string::String, sync::Arc};

pub(crate) use crate::run_queue::{AxRunQueue, RUN_QUEUE};

#[doc(cfg(feature = "multitask"))]
pub use crate::task::{CurrentTask, TaskId, TaskInner};
#[cfg(not(feature = "musl"))]
use crate::tsd;
#[doc(cfg(feature = "multitask"))]
pub use crate::wait_queue::{WaitQueue, WaitQueueWithMetadata};

/// The reference type of a task.
pub type AxTaskRef = Arc<AxTask>;

cfg_if::cfg_if! {
    if #[cfg(feature = "sched_rr")] {
        const MAX_TIME_SLICE: usize = 5;
        pub(crate) type AxTask = scheduler::RRTask<TaskInner, MAX_TIME_SLICE>;
        pub(crate) type Scheduler = scheduler::RRScheduler<TaskInner, MAX_TIME_SLICE>;
    } else if #[cfg(feature = "sched_cfs")] {
        pub(crate) type AxTask = scheduler::CFSTask<TaskInner>;
        pub(crate) type Scheduler = scheduler::CFScheduler<TaskInner>;
    } else {
        // If no scheduler features are set, use FIFO as the default.
        pub(crate) type AxTask = scheduler::FifoTask<TaskInner>;
        pub(crate) type Scheduler = scheduler::FifoScheduler<TaskInner>;
    }
}

#[cfg(feature = "preempt")]
struct KernelGuardIfImpl;

#[cfg(feature = "preempt")]
#[crate_interface::impl_interface]
impl kernel_guard::KernelGuardIf for KernelGuardIfImpl {
    fn disable_preempt() {
        if let Some(curr) = current_may_uninit() {
            curr.disable_preempt();
        }
    }

    fn enable_preempt() {
        if let Some(curr) = current_may_uninit() {
            curr.enable_preempt(true);
        }
    }
}

/// Gets the current task, or returns [`None`] if the current task is not
/// initialized.
pub fn current_may_uninit() -> Option<CurrentTask> {
    CurrentTask::try_get()
}

/// Gets the current task.
///
/// # Panics
///
/// Panics if the current task is not initialized.
#[inline(never)]
pub fn current() -> CurrentTask {
    CurrentTask::get()
}

/// Initializes the task scheduler (for the primary CPU).
pub fn init_scheduler() {
    info!("Initialize scheduling...");

    crate::run_queue::init();
    #[cfg(feature = "irq")]
    crate::timers::init();
    #[cfg(not(feature = "musl"))]
    tsd::init();

    info!("  use {} scheduler.", Scheduler::scheduler_name());
}

/// Initializes the task scheduler for secondary CPUs.
pub fn init_scheduler_secondary() {
    crate::run_queue::init_secondary();
}

/// Handles periodic timer ticks for the task manager.
///
/// For example, advance scheduler states, checks timed events, etc.
#[cfg(feature = "irq")]
#[doc(cfg(feature = "irq"))]
pub fn on_timer_tick() {
    crate::timers::check_events();
    RUN_QUEUE.lock().scheduler_timer_tick();
}

/// Spawns a new task with the given parameters.
///
/// Returns the task reference.
pub fn spawn_raw<F>(f: F, name: String, stack_size: usize) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    let task = TaskInner::new(f, name, stack_size);
    RUN_QUEUE.lock().add_task(task.clone());
    task
}

/// Used by musl
#[cfg(feature = "musl")]
pub fn pspawn_raw<F>(
    f: F,
    name: String,
    stack_size: usize,
    tls: usize,
    set_tid: core::sync::atomic::AtomicU64,
    tl: core::sync::atomic::AtomicU64,
) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    TaskInner::new_musl(f, name, stack_size, tls, set_tid, tl)
}

// temporarily only support aarch64
#[cfg(all(
    any(target_arch = "aarch64", target_arch = "riscv64"),
    feature = "paging",
    feature = "fs"
))]
pub fn fork_task() -> Option<AxTaskRef> {
    use core::mem::ManuallyDrop;

    let current_id = current().id().as_u64();
    let children_process = TaskInner::fork();

    // Judge whether the parent process is blocked, if yes, add it to the blocking queue of the child process
    if current().id().as_u64() == current_id {
        RUN_QUEUE.lock().add_task(children_process.clone());

        return Some(children_process);
    }

    unsafe {
        RUN_QUEUE.force_unlock();
    }

    // should not drop the children_process here, because it will be taken in the parent process
    // and dropped in the parent process
    let _ = ManuallyDrop::new(children_process);

    #[cfg(feature = "irq")]
    ruxhal::arch::enable_irqs();

    None
}

/// Spawns a new task with the default parameters.
///
/// The default task name is an empty string. The default task stack size is
/// [`ruxconfig::TASK_STACK_SIZE`].
///
/// Returns the task reference.
pub fn spawn<F>(f: F) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    spawn_raw(f, "".into(), ruxconfig::TASK_STACK_SIZE)
}

/// Used by musl
#[cfg(feature = "musl")]
pub fn pspawn<F>(
    f: F,
    tls: usize,
    set_tid: core::sync::atomic::AtomicU64,
    tl: core::sync::atomic::AtomicU64,
) -> AxTaskRef
where
    F: FnOnce() + Send + 'static,
{
    pspawn_raw(f, "".into(), ruxconfig::TASK_STACK_SIZE, tls, set_tid, tl)
}

/// Used by musl
///
/// Put new thread into run_queue
pub fn put_task(task: AxTaskRef) {
    RUN_QUEUE.lock().add_task(task);
}

/// Set the priority for current task.
///
/// The range of the priority is dependent on the underlying scheduler. For
/// example, in the [CFS] scheduler, the priority is the nice value, ranging from
/// -20 to 19.
///
/// Returns `true` if the priority is set successfully.
///
/// [CFS]: https://en.wikipedia.org/wiki/Completely_Fair_Scheduler
pub fn set_priority(prio: isize) -> bool {
    RUN_QUEUE.lock().set_current_priority(prio)
}

/// Current task gives up the CPU time voluntarily, and switches to another
/// ready task.
pub fn yield_now() {
    RUN_QUEUE.lock().yield_current();
}

#[cfg(feature = "fs")]
struct SchedYieldIfImpl;

#[cfg(feature = "fs")]
#[crate_interface::impl_interface]
impl ruxfs::fifo::SchedYieldIf for SchedYieldIfImpl {
    fn yield_now() {
        yield_now();
    }
}

/// Current task is going to sleep for the given duration.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(ruxhal::time::current_time() + dur);
}

/// Current task is going to sleep, it will be woken up at the given deadline.
///
/// If the feature `irq` is not enabled, it uses busy-wait instead.
pub fn sleep_until(deadline: ruxhal::time::TimeValue) {
    #[cfg(feature = "irq")]
    RUN_QUEUE.lock().sleep_until(deadline);
    #[cfg(not(feature = "irq"))]
    ruxhal::time::busy_wait_until(deadline);
}

/// Exits the current task.
pub fn exit(exit_code: i32) -> ! {
    #[cfg(not(feature = "musl"))]
    current().destroy_keys();
    RUN_QUEUE.lock().exit_current(exit_code)
}

/// The idle task routine.
///
/// It runs an infinite loop that keeps calling [`yield_now()`].
pub fn run_idle() -> ! {
    loop {
        yield_now();
        debug!(
            "idle task[{}]: waiting for IRQs...",
            current().id().as_u64()
        );
        #[cfg(feature = "irq")]
        ruxhal::arch::wait_for_irqs();
    }
}
