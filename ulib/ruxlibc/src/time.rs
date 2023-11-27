/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use ruxos_posix_api::{sys_clock_gettime, sys_clock_settime, sys_nanosleep};
#[cfg(feature = "signal")]
use ruxos_posix_api::{sys_getitimer, sys_setitimer};

use crate::{ctypes, utils::e};

/// Get clock time since booting
#[no_mangle]
pub unsafe extern "C" fn clock_gettime(clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    e(sys_clock_gettime(clk, ts))
}

/// Set clock time since booting
#[no_mangle]
pub unsafe extern "C" fn clock_settime(clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    e(sys_clock_settime(clk, ts))
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
#[no_mangle]
pub unsafe extern "C" fn nanosleep(
    req: *const ctypes::timespec,
    rem: *mut ctypes::timespec,
) -> c_int {
    e(sys_nanosleep(req, rem))
}

/// Set timer to send signal after some time
#[no_mangle]
pub unsafe extern "C" fn setitimer(
    _which: c_int,
    _new: *const ctypes::itimerval,
    _old: *mut ctypes::itimerval,
) -> c_int {
    #[cfg(feature = "signal")]
    {
        if !_old.is_null() {
            let res = e(sys_getitimer(_which, _old));
            if res != 0 {
                return res;
            }
        }
        e(sys_setitimer(_which, _new))
    }
    #[cfg(not(feature = "signal"))]
    {
        e(0)
    }
}

/// Set timer to send signal after some time
#[no_mangle]
pub unsafe extern "C" fn getitimer(_which: c_int, _curr_value: *mut ctypes::itimerval) -> c_int {
    #[cfg(feature = "signal")]
    {
        e(sys_getitimer(_which, _curr_value))
    }
    #[cfg(not(feature = "signal"))]
    {
        e(0)
    }
}
