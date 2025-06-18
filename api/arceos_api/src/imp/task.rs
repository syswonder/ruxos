/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

pub fn ax_sleep_until(deadline: crate::time::AxTimeValue) {
    #[cfg(feature = "multitask")]
    ruxtask::sleep_until(deadline);
    #[cfg(not(feature = "multitask"))]
    ruxhal::time::busy_wait_until(deadline);
}

pub fn ax_yield_now() {
    #[cfg(feature = "multitask")]
    ruxtask::yield_now();
    #[cfg(not(feature = "multitask"))]
    if cfg!(feature = "irq") {
        ruxhal::arch::wait_for_irqs();
    } else {
        core::hint::spin_loop();
    }
}

pub fn ax_exit(_exit_code: i32) -> ! {
    #[cfg(feature = "multitask")]
    ruxtask::exit(_exit_code);
    #[cfg(not(feature = "multitask"))]
    ruxhal::misc::terminate();
}

cfg_task! {
    use core::time::Duration;

    /// A handle to a task.
    pub struct AxTaskHandle {
        inner: ruxtask::AxTaskRef,
        id: u64,
    }

    impl AxTaskHandle {
        /// Returns the task ID.
        pub fn id(&self) -> u64 {
            self.id
        }
    }

    /// A handle to a wait queue.
    ///
    /// A wait queue is used to store sleeping tasks waiting for a certain event
    /// to happen.
    pub struct AxWaitQueueHandle(ruxtask::WaitQueue);

    impl AxWaitQueueHandle {
        /// Creates a new empty wait queue.
        pub const fn new() -> Self {
            Self(ruxtask::WaitQueue::new())
        }
    }

    impl Default for AxWaitQueueHandle {
        fn default() -> Self {
            Self::new()
        }
    }

    pub fn ax_current_task_id() -> u64 {
        ruxtask::current().id().as_u64()
    }

    pub fn ax_spawn<F>(f: F, name: alloc::string::String, stack_size: usize) -> AxTaskHandle
    where
        F: FnOnce() + Send + 'static,
    {
        let inner = ruxtask::spawn_raw(f, name, stack_size);
        AxTaskHandle {
            id: inner.id().as_u64(),
            inner,
        }
    }

    pub fn ax_wait_for_exit(task: AxTaskHandle) -> Option<i32> {
        task.inner.join()
    }

    pub fn ax_set_current_priority(prio: isize) -> crate::AxResult {
        if ruxtask::set_priority(prio) {
            Ok(())
        } else {
            axerrno::ax_err!(
                BadState,
                "ax_set_current_priority: failed to set task priority"
            )
        }
    }

    pub fn ax_wait_queue_wait(
        wq: &AxWaitQueueHandle,
        until_condition: impl Fn() -> bool,
        timeout: Option<Duration>,
    ) -> bool {
        #[cfg(feature = "irq")]
        if let Some(dur) = timeout {
            return wq.0.wait_timeout_until(dur, until_condition);
        }

        if timeout.is_some() {
            axlog::warn!("ax_wait_queue_wait: the `timeout` argument is ignored without the `irq` feature");
        }
        wq.0.wait_until(until_condition);
        false
    }

    pub fn ax_wait_queue_wake(wq: &AxWaitQueueHandle, count: u32) {
        if count == u32::MAX {
            wq.0.notify_all(true);
        } else {
            for _ in 0..count {
                wq.0.notify_one(true);
            }
        }
    }
}
