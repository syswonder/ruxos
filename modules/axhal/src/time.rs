/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Time-related operations.

pub use core::time::Duration;

/// A measurement of the system clock.
///
/// Currently, it reuses the [`core::time::Duration`] type. But it does not
/// represent a duration, but a clock time.
pub type TimeValue = Duration;

#[cfg(feature = "irq")]
pub use crate::platform::irq::TIMER_IRQ_NUM;
#[cfg(feature = "irq")]
pub use crate::platform::time::set_oneshot_timer;
pub use crate::platform::time::{current_ticks, nanos_to_ticks, ticks_to_nanos};
#[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
#[cfg(feature = "rtc")]
pub use crate::platform::time::{rtc_read_time, rtc_write_time};

/// Number of milliseconds in a second.
pub const MILLIS_PER_SEC: u64 = 1_000;
/// Number of microseconds in a second.
pub const MICROS_PER_SEC: u64 = 1_000_000;
/// Number of nanoseconds in a second.
pub const NANOS_PER_SEC: u64 = 1_000_000_000;
/// Number of nanoseconds in a millisecond.
pub const NANOS_PER_MILLIS: u64 = 1_000_000;
/// Number of nanoseconds in a microsecond.
pub const NANOS_PER_MICROS: u64 = 1_000;

/// Returns the current clock time in nanoseconds.
pub fn current_time_nanos() -> u64 {
    ticks_to_nanos(current_ticks())
}

/// Returns the current clock time in [`TimeValue`].
#[allow(unreachable_code)]
pub fn current_time() -> TimeValue {
    #[cfg(any(target_arch = "x86_64", target_arch = "aarch64"))]
    #[cfg(feature = "rtc")]
    {
        let nanos = current_time_nanos();
        let rtc_time = rtc_read_time();
        return Duration::new(rtc_time, (nanos % (NANOS_PER_SEC)) as u32);
    }
    TimeValue::from_nanos(current_time_nanos())
}

/// set time value
pub fn set_current_time(_new_tv: TimeValue) {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    #[cfg(feature = "rtc")]
    rtc_write_time(_new_tv.as_secs() as u32);
}

/// Busy waiting for the given duration.
pub fn busy_wait(dur: Duration) {
    busy_wait_until(current_time() + dur);
}

/// Busy waiting until reaching the given deadline.
pub fn busy_wait_until(deadline: TimeValue) {
    while current_time() < deadline {
        core::hint::spin_loop();
    }
}
