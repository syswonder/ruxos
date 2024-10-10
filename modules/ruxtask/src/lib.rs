/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos](https://github.com/syswonder/ruxos) task management module.
//!
//! This module provides primitives for task management, including task
//! creation, scheduling, sleeping, termination, etc. The scheduler algorithm
//! is configurable by cargo features.
//!
//! # Cargo Features
//!
//! - `multitask`: Enable multi-task support. If it's enabled, complex task
//!   management and scheduling is used, as well as more task-related APIs.
//!   Otherwise, only a few APIs with naive implementation is available.
//! - `irq`: Interrupts are enabled. If this feature is enabled, timer-based
//!    APIs can be used, such as [`sleep`], [`sleep_until`], and
//!    [`WaitQueue::wait_timeout`].
//! - `preempt`: Enable preemptive scheduling.
//! - `sched_fifo`: Use the [FIFO cooperative scheduler][1]. It also enables the
//!   `multitask` feature if it is enabled. This feature is enabled by default,
//!   and it can be overriden by other scheduler features.
//! - `sched_rr`: Use the [Round-robin preemptive scheduler][2]. It also enables
//!   the `multitask` and `preempt` features if it is enabled.
//! - `sched_cfs`: Use the [Completely Fair Scheduler][3]. It also enables the
//!   the `multitask` and `preempt` features if it is enabled.
//!
//! [1]: scheduler::FifoScheduler
//! [2]: scheduler::RRScheduler
//! [3]: scheduler::CFScheduler

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

cfg_if::cfg_if! {
    if #[cfg(feature = "multitask")] {
        #[macro_use]
        extern crate log;
        extern crate alloc;

        mod run_queue;
        pub mod task;
        mod api;
        mod wait_queue;
        #[cfg(feature = "paging")]
        pub mod vma;
        // #[cfg(feature = "fs")]
        pub mod fs;
        #[cfg(feature = "irq")]
        /// load average
        pub mod loadavg;
        /// specific key-value storage for each task
        #[cfg(not(feature = "musl"))]
        pub mod tsd;
        /// TODO: if irq is disabled, what value should AVENRUN be?
        /// average run load, same as in linux kernel
        static mut AVENRUN: [u64; 3] = [0, 0, 0];

        /// Get the load average
        pub fn get_avenrun(loads: &mut [u64; 3]) {
            for i in 0..3 {
                unsafe {
                    // TODO: disable irq for safety
                    loads[i] = AVENRUN[i];
                }
            }
        }

        #[cfg(feature = "irq")]
        mod timers;

        #[doc(cfg(feature = "multitask"))]
        pub use self::api::*;
        pub use self::api::{sleep, sleep_until, yield_now};
        pub use task::TaskState;
    } else {
        mod api_s;
        pub use self::api_s::{sleep, sleep_until, yield_now};
    }
}
