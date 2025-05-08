/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_int, c_long};
use core::time::Duration;

use crate::ctypes;

use axerrno::LinuxError;

// nanoseconds per a second
const NANO_PER_SECOND: i64 = 1000000000;

impl From<ctypes::timespec> for Duration {
    fn from(ts: ctypes::timespec) -> Self {
        Duration::new(ts.tv_sec as u64, ts.tv_nsec as u32)
    }
}

impl From<ctypes::timeval> for Duration {
    fn from(tv: ctypes::timeval) -> Self {
        Duration::new(tv.tv_sec as u64, tv.tv_usec as u32 * 1000)
    }
}

impl From<Duration> for ctypes::timespec {
    fn from(d: Duration) -> Self {
        ctypes::timespec {
            tv_sec: d.as_secs() as c_long,
            tv_nsec: d.subsec_nanos() as c_long,
        }
    }
}

impl From<Duration> for ctypes::timeval {
    fn from(d: Duration) -> Self {
        ctypes::timeval {
            tv_sec: d.as_secs() as c_long,
            tv_usec: d.subsec_micros() as c_long,
        }
    }
}

/// Get clock time since booting
pub unsafe fn sys_clock_gettime(_clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_gettime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let now = ruxhal::time::current_time().into();
        unsafe { *ts = now };
        debug!("sys_clock_gettime: {}.{:09}s", now.tv_sec, now.tv_nsec);
        Ok(0)
    })
}

/// Get clock time since booting
pub unsafe fn sys_clock_settime(_clk: ctypes::clockid_t, ts: *const ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_setttime, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let new_tv = Duration::from(*ts);
        debug!(
            "sys_clock_setttime: {}.{:09}s",
            new_tv.as_secs(),
            new_tv.as_nanos()
        );
        ruxhal::time::set_current_time(new_tv);
        Ok(0)
    })
}

/// Return the resolution (precision) of a specified clock `clk_id`.
/// TODO: Currently we only have one simple clock source in the OS.
/// We ignore `clk_id` and always return a fixed resolution of 1ns.
/// In the future, we may:
///  - Distinguish different clock IDs (e.g., REALTIME vs MONOTONIC).
///  - Return a more realistic resolution based on hardware capabilities.
pub unsafe fn sys_clock_getres(clk_id: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_clock_getres, {
        if ts.is_null() {
            return Err(LinuxError::EFAULT);
        }
        (*ts).tv_sec = 0;
        (*ts).tv_nsec = 1;
        debug!("sys_clock_getres: clk_id={}, returning 0s + 1ns", clk_id);
        Ok(0)
    })
}

/// Sleep until some nanoseconds
///
/// TODO: should be woken by signals, and set errno
/// TODO: deal with flags
pub unsafe fn sys_clock_nanosleep(
    _which_clock: ctypes::clockid_t,
    _flags: c_int,
    req: *const ctypes::timespec,
    rem: *mut ctypes::timespec,
) -> c_int {
    syscall_body!(sys_clock_nanosleep, {
        unsafe {
            if req.is_null() || (*req).tv_nsec < 0 || (*req).tv_nsec >= NANO_PER_SECOND {
                return Err(LinuxError::EINVAL);
            }
        }

        let deadline = unsafe { Duration::from(*req) };

        let now = ruxhal::time::current_time();

        if now >= deadline {
            return Ok(0);
        }

        #[cfg(feature = "multitask")]
        ruxtask::sleep_until(deadline);
        #[cfg(not(feature = "multitask"))]
        ruxhal::time::busy_wait_until(deadline);

        let after = ruxhal::time::current_time();
        let actual = after - now;
        let due = deadline - now;

        if let Some(diff) = due.checked_sub(actual) {
            if !rem.is_null() {
                unsafe { (*rem) = diff.into() };
            }
            return Err(LinuxError::EINTR);
        }
        Ok(0)
    })
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
pub unsafe fn sys_nanosleep(req: *const ctypes::timespec, rem: *mut ctypes::timespec) -> c_int {
    syscall_body!(sys_nanosleep, {
        unsafe {
            if req.is_null() || (*req).tv_nsec < 0 || (*req).tv_nsec >= NANO_PER_SECOND {
                return Err(LinuxError::EINVAL);
            }
        }

        let dur = unsafe {
            debug!("sys_nanosleep <= {}.{:09}s", (*req).tv_sec, (*req).tv_nsec);
            Duration::from(*req)
        };

        let now = ruxhal::time::current_time();

        #[cfg(feature = "multitask")]
        ruxtask::sleep(dur);
        #[cfg(not(feature = "multitask"))]
        ruxhal::time::busy_wait(dur);

        let after = ruxhal::time::current_time();
        let actual = after - now;

        if let Some(diff) = dur.checked_sub(actual) {
            if !rem.is_null() {
                unsafe { (*rem) = diff.into() };
            }
            return Err(LinuxError::EINTR);
        }
        Ok(0)
    })
}

/// Get time of the day, ignore second parameter
pub unsafe fn sys_gettimeofday(ts: *mut ctypes::timespec, flags: c_int) -> c_int {
    debug!("sys_gettimeofday <= flags: {}", flags);
    unsafe { sys_clock_gettime(0, ts) }
}

/// TODO: get process and waited-for child process times
pub unsafe fn sys_times(_buf: *mut usize) -> c_int {
    syscall_body!(sys_times, Ok(0))
}
