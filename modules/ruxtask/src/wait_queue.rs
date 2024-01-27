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
use spinlock::SpinRaw;

use crate::{AxRunQueue, AxTaskRef, CurrentTask, RUN_QUEUE};

type ItemType<Meta> = (AxTaskRef, Meta);
type QueueType<Meta> = VecDeque<ItemType<Meta>>;

/// A queue to store sleeping tasks, each with its metadata.
///
/// # Examples
///
/// ```
/// use ruxtask::WaitQueue;
/// use core::sync::atomic::{AtomicU32, Ordering};
///
/// static VALUE: AtomicU32 = AtomicU32::new(0);
/// static WQ: WaitQueue = WaitQueue::new();
///
/// ruxtask::init_scheduler();
/// // spawn a new task that updates `VALUE` and notifies the main task
/// ruxtask::spawn(|| {
///     assert_eq!(VALUE.load(Ordering::Relaxed), 0);
///     VALUE.fetch_add(1, Ordering::Relaxed);
///     WQ.notify_one(true); // wake up the main task
/// });
///
/// WQ.wait(); // block until `notify()` is called
/// assert_eq!(VALUE.load(Ordering::Relaxed), 1);
/// ```
pub struct WaitQueueWithMetadata<Meta> {
    queue: SpinRaw<QueueType<Meta>>, // we already disabled IRQs when lock the `RUN_QUEUE`
}

/// A wait queue with no metadata.
pub type WaitQueue = WaitQueueWithMetadata<()>;

impl<Meta> WaitQueueWithMetadata<Meta> {
    /// Creates an empty wait queue.
    pub const fn new() -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::new()),
        }
    }

    /// Creates an empty wait queue with space for at least `capacity` elements.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: SpinRaw::new(VecDeque::with_capacity(capacity)),
        }
    }

    fn cancel_events(&self, curr: CurrentTask) {
        Self::cancel_events_locked(&mut self.queue.lock(), curr);
    }

    fn cancel_events_locked(queue: &mut QueueType<Meta>, curr: CurrentTask) {
        // A task can be wake up only one events (timer or `notify()`), remove
        // the event from another queue.
        if curr.in_wait_queue() {
            // wake up by timer (timeout).
            // `RUN_QUEUE` is not locked here, so disable IRQs.
            let _guard = kernel_guard::IrqSave::new();
            queue.retain(|(t, _)| !curr.ptr_eq(t));
            curr.set_in_wait_queue(false);
        }
        #[cfg(feature = "irq")]
        if curr.in_timer_list() {
            // timeout was set but not triggered (wake up by `WaitQueue::notify()`)
            crate::timers::cancel_alarm(curr.as_task_ref());
        }
    }

    /// Blocks the current task and put it into the wait queue, until other task
    /// notifies it.
    pub fn wait_meta(&self, meta: Meta) {
        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back((task, meta))
        });
        self.cancel_events(crate::current());
    }

    /// If `condition` returns [`Ok`], blocks the current task and put it into the wait queue,
    /// until other task notifies it.
    pub fn wait_meta_if<F, R>(&self, meta: Meta, mut condition: F) -> Result<(), R>
    where
        F: FnMut() -> Result<(), R>,
    {
        let mut rq = RUN_QUEUE.lock();
        let mut wq = self.queue.lock();
        condition()?;

        rq.block_current(|task| {
            task.set_in_wait_queue(true);
            wq.push_back((task, meta));
            drop(wq);
        });
        self.cancel_events(crate::current());

        Ok(())
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_meta(&self, dur: core::time::Duration, meta: Meta) -> bool {
        let deadline = dur + ruxhal::time::current_time();
        self.wait_timeout_absolutely_meta(deadline, meta)
    }

    /// If `condition` returns [`Ok`], blocks the current task and put it into the wait queue,
    /// until other tasks notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_meta_if<F, R>(
        &self,
        dur: core::time::Duration,
        meta: Meta,
        condition: F,
    ) -> Result<bool, R>
    where
        F: FnMut() -> Result<(), R>,
    {
        let deadline = dur + ruxhal::time::current_time();
        self.wait_timeout_absolutely_meta_if(deadline, meta, condition)
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given deadline has elapsed.
    pub fn wait_timeout_absolutely_meta(&self, deadline: core::time::Duration, meta: Meta) -> bool {
        let curr = crate::current();
        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        #[cfg(feature = "irq")]
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        RUN_QUEUE.lock().block_current(|task| {
            task.set_in_wait_queue(true);
            self.queue.lock().push_back((task, meta))
        });
        let timeout = curr.in_wait_queue(); // still in the wait queue, must have timed out
        self.cancel_events(curr);
        timeout
    }

    /// If `condition` returns [`Ok`], blocks the current task and put it into the wait queue,
    /// until other tasks notify it, or the given deadline has elapsed.
    pub fn wait_timeout_absolutely_meta_if<F, R>(
        &self,
        deadline: core::time::Duration,
        meta: Meta,
        mut condition: F,
    ) -> Result<bool, R>
    where
        F: FnMut() -> Result<(), R>,
    {
        let curr = crate::current();
        let mut rq = RUN_QUEUE.lock();
        let mut wq = self.queue.lock();
        condition()?;

        debug!(
            "task wait_timeout: {} deadline={:?}",
            curr.id_name(),
            deadline
        );
        #[cfg(feature = "irq")]
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        rq.block_current(|task| {
            task.set_in_wait_queue(true);
            wq.push_back((task, meta));
            drop(wq);
        });
        let timeout = curr.in_wait_queue(); // still in the wait queue, must have timed out
        self.cancel_events(curr);
        Ok(timeout)
    }

    /// Wakes up one task in the wait queue, usually the first one.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_one(&self, resched: bool) -> bool {
        let mut rq = RUN_QUEUE.lock();
        if !self.queue.lock().is_empty() {
            self.notify_one_locked(resched, &mut rq)
        } else {
            false
        }
    }

    /// Wakes all tasks in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_all(&self, resched: bool) {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if let Some((task, _)) = self.queue.lock().pop_front() {
                task.set_in_wait_queue(false);
                rq.unblock_task(task, resched);
            } else {
                break;
            }
            drop(rq); // we must unlock `RUN_QUEUE` after unlocking `self.queue`.
        }
    }

    /// Wake up the given task in the wait queue.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task(&self, resched: bool, task: &AxTaskRef) -> bool {
        let mut rq = RUN_QUEUE.lock();
        let mut wq = self.queue.lock();
        if let Some(index) = wq.iter().position(|(t, _)| Arc::ptr_eq(t, task)) {
            task.set_in_wait_queue(false);
            rq.unblock_task(wq.remove(index).unwrap().0, resched);
            true
        } else {
            false
        }
    }

    /// Wake up all corresponding tasks that `filter` returns true.
    ///
    /// Returns number of tasks awaken.
    ///
    /// If `resched` is true, the current task will be preempted when the
    /// preemption is enabled.
    pub fn notify_task_if<F>(&self, resched: bool, mut filter: F) -> usize
    where
        F: FnMut(&AxTaskRef, &Meta) -> bool,
    {
        let mut rq = RUN_QUEUE.lock();
        let mut wq = self.queue.lock();
        let len_before = wq.len();

        wq.retain(|(task, meta)| {
            if filter(task, meta) {
                task.set_in_wait_queue(false);
                rq.unblock_task(task.clone(), resched);
                false
            } else {
                true
            }
        });

        len_before - wq.len()
    }

    pub(crate) fn notify_one_locked(&self, resched: bool, rq: &mut AxRunQueue) -> bool {
        if let Some((task, _)) = self.queue.lock().pop_front() {
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
            true
        } else {
            false
        }
    }

    pub(crate) fn notify_all_locked(&self, resched: bool, rq: &mut AxRunQueue) {
        while let Some((task, _)) = self.queue.lock().pop_front() {
            task.set_in_wait_queue(false);
            rq.unblock_task(task, resched);
        }
    }

    /// Queue a given task with its metadata given.
    ///
    /// It is marked as unsafe as it does nothing other than queueing the task,
    /// so one might want to modify the state of the task appropriately before/after preforming
    /// this operation.
    ///
    /// # Safety
    ///This function does not cause memory/concurrent safety issues directly, but may lead
    /// to resource leak or break assumptions in scheduler when used incautiously.
    pub unsafe fn queue_task_meta(&self, task: AxTaskRef, meta: Meta) {
        self.queue.lock().push_back((task, meta));
    }

    /// Dequeue the given task, returning its metadata if this task is actually inside this queue.
    ///
    /// It is marked as unsafe as it does nothing other than dequeueing the task,
    /// so one might want to modify the state of the task appropriately before/after preforming
    /// this operation.
    ///
    /// # Safety
    ///This function does not cause memory/concurrent safety issues directly, but may lead
    /// to resource leak or break assumptions in scheduler when used incautiously.
    pub unsafe fn dequeue_task(&self, task: &AxTaskRef) -> Option<Meta> {
        let mut wq = self.queue.lock();
        let pos = wq.iter().position(|(t, _)| Arc::ptr_eq(t, task));
        if let Some(pos) = pos {
            let (_, meta) = wq.swap_remove_back(pos).unwrap();
            Some(meta)
        } else {
            None
        }
    }

    /// Dequeue all corresponding tasks that `filter` returns true.
    ///
    /// It is marked as unsafe as it does nothing other than dequeueing the tasks,
    /// so one might want to modify the state of the task appropriately before/after preforming
    /// this operation.
    ///
    /// # Safety
    ///This function does not cause memory/concurrent safety issues directly, but may lead
    /// to resource leak or break assumptions in scheduler when used incautiously.
    pub unsafe fn dequeue_tasks_if<F, Op>(&self, mut filter: F, mut dequeue_op: Op)
    where
        F: FnMut(&AxTaskRef, &Meta) -> bool,
        Op: FnMut(AxTaskRef, Meta),
    {
        let mut wq = self.queue.lock();
        let first_false = partition_deque(&mut wq, |(t, m)| filter(t, m));

        // `..first_false` represents the range that `filter` returns true for all tasks within.
        wq.drain(..first_false).for_each(|(t, m)| dequeue_op(t, m));
    }
}

impl<Meta: Clone> WaitQueueWithMetadata<Meta> {
    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the condition becomes true.
    pub fn wait_until_meta<F>(&self, mut condition: F, meta: Meta)
    where
        F: FnMut() -> bool,
    {
        loop {
            let mut rq = RUN_QUEUE.lock();
            if condition() {
                break;
            }
            rq.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back((task, meta.clone()));
            });
        }
        self.cancel_events(crate::current());
    }

    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true, or the given duration has elapsed.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the above conditions are met.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_until_meta<F>(
        &self,
        dur: core::time::Duration,
        mut condition: F,
        meta: Meta,
    ) -> bool
    where
        F: FnMut() -> bool,
    {
        let curr = crate::current();
        let deadline = ruxhal::time::current_time() + dur;
        debug!(
            "task wait_timeout: {}, deadline={:?}",
            curr.id_name(),
            deadline
        );
        crate::timers::set_alarm_wakeup(deadline, curr.clone());

        let mut timeout = true;
        while ruxhal::time::current_time() < deadline {
            let mut rq = RUN_QUEUE.lock();
            if condition() {
                timeout = false;
                break;
            }
            rq.block_current(|task| {
                task.set_in_wait_queue(true);
                self.queue.lock().push_back((task, meta.clone()));
            });
        }
        self.cancel_events(curr);
        timeout
    }
}

impl<Meta: Default> WaitQueueWithMetadata<Meta> {
    /// Blocks the current task and put it into the wait queue, until other task
    /// notifies it.
    pub fn wait(&self) {
        self.wait_meta(Default::default())
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given duration has elapsed.
    #[cfg(feature = "irq")]
    pub fn wait_timeout(&self, dur: core::time::Duration) -> bool {
        self.wait_timeout_meta(dur, Default::default())
    }

    /// Blocks the current task and put it into the wait queue, until other tasks
    /// notify it, or the given deadling has elapsed.
    pub fn wait_timeout_absolutely(&self, deadline: core::time::Duration) -> bool {
        self.wait_timeout_absolutely_meta(deadline, Default::default())
    }

    /// Queue a given task with "default" metadata.
    ///
    /// It is marked as unsafe as it does nothing other than queueing the task,
    /// so one might want to modify the state of the task appropriately before/after preforming
    /// this operation.
    ///
    /// # Safety
    /// This function does not cause memory/concurrent safety issues directly, but may lead
    /// to resource leak or break assumptions in scheduler when used incautiously.
    pub unsafe fn queue_task(&self, task: AxTaskRef) {
        self.queue.lock().push_back((task, Default::default()));
    }
}

impl<Meta: Default + Clone> WaitQueueWithMetadata<Meta> {
    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the condition becomes true.
    pub fn wait_until<F>(&self, condition: F)
    where
        F: FnMut() -> bool,
    {
        self.wait_until_meta(condition, Default::default())
    }

    /// Blocks the current task and put it into the wait queue, until the given
    /// `condition` becomes true, or the given duration has elapsed.
    ///
    /// Note that even other tasks notify this task, it will not wake up until
    /// the above conditions are met.
    #[cfg(feature = "irq")]
    pub fn wait_timeout_until<F>(&self, dur: core::time::Duration, condition: F) -> bool
    where
        F: FnMut() -> bool,
    {
        self.wait_timeout_until_meta(dur, condition, Default::default())
    }
}

/// Partition a [`VecDeque`] in-place so that it contains all elements for
/// which `predicate(e)` is `true`, followed by all elements for which
/// `predicate(e)` is `false`.
/// Returns the index of the first element which returned false.
/// Returns 0 if all elements returned false.
/// Returns `data.len()` if all elements returned true.
fn partition_deque<T, F>(data: &mut VecDeque<T>, mut pred: F) -> usize
where
    F: FnMut(&T) -> bool,
{
    let len = data.len();
    if len == 0 {
        return 0;
    }
    let (mut l, mut r) = (0, len - 1);
    loop {
        while l < len && pred(&data[l]) {
            l += 1;
        }
        while r > 0 && !pred(&data[r]) {
            r -= 1;
        }
        if l >= r {
            return l;
        }
        data.swap(l, r);
    }
}
