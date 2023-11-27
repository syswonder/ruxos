/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Native threads.

#[cfg(feature = "multitask")]
mod multi;
#[cfg(feature = "multitask")]
pub use multi::*;

use arceos_api::task as api;

/// Current thread gives up the CPU time voluntarily, and switches to another
/// ready thread.
///
/// For single-threaded configuration (`multitask` feature is disabled), we just
/// relax the CPU and wait for incoming interrupts.
pub fn yield_now() {
    api::ax_yield_now();
}

/// Exits the current thread.
///
/// For single-threaded configuration (`multitask` feature is disabled),
/// it directly terminates the main thread and shutdown.
pub fn exit(exit_code: i32) -> ! {
    api::ax_exit(exit_code);
}

/// Current thread is going to sleep for the given duration.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep(dur: core::time::Duration) {
    sleep_until(arceos_api::time::ax_current_time() + dur);
}

/// Current thread is going to sleep, it will be woken up at the given deadline.
///
/// If one of `multitask` or `irq` features is not enabled, it uses busy-wait
/// instead.
pub fn sleep_until(deadline: arceos_api::time::AxTimeValue) {
    api::ax_sleep_until(deadline);
}
