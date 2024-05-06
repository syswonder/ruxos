/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use ruxos_posix_api::{sys_exit, sys_getpid, sys_gettid};
#[cfg(feature = "signal")]
use {
    crate::getitimer,
    crate::{ctypes, utils::e},
    core::ffi::c_uint,
    ruxos_posix_api::sys_setitimer,
};

/// Get current thread ID.
#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid()
}

/// Get current thread ID.
#[no_mangle]
pub unsafe extern "C" fn gettid() -> c_int {
    sys_gettid()
}

/// Abort the current process.
#[no_mangle]
pub unsafe extern "C" fn abort() -> ! {
    panic!()
}

/// Exits the current thread.
#[no_mangle]
pub unsafe extern "C" fn exit(exit_code: c_int) -> ! {
    sys_exit(exit_code)
}

/// Set an alarm clock for delivery of a signal
#[cfg(feature = "signal")]
#[no_mangle]
pub unsafe extern "C" fn alarm(seconds: c_uint) -> c_uint {
    let it = ctypes::itimerval {
        it_interval: ctypes::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
        it_value: ctypes::timeval {
            tv_sec: seconds as i64,
            tv_usec: 0,
        },
    };
    let mut old = ctypes::itimerval::default();
    if getitimer(ctypes::ITIMER_REAL as c_int, &mut old) < 0 {
        e(sys_setitimer(ctypes::ITIMER_REAL as c_int, &it)) as c_uint
    } else {
        old.it_value.tv_sec as c_uint
    }
}

/// Schedule signal after given number of microseconds
#[cfg(feature = "signal")]
#[no_mangle]
pub unsafe extern "C" fn ualarm(useconds: c_uint, interval: c_uint) -> c_uint {
    let it = ctypes::itimerval {
        it_interval: ctypes::timeval {
            tv_sec: 0,
            tv_usec: interval as i64,
        },
        it_value: ctypes::timeval {
            tv_sec: 0,
            tv_usec: useconds as i64,
        },
    };
    let mut old = ctypes::itimerval::default();
    if getitimer(ctypes::ITIMER_REAL as i32, &mut old) < 0 {
        e(sys_setitimer(ctypes::ITIMER_REAL as i32, &it));
        0
    } else {
        core::time::Duration::from(old.it_value).as_micros() as c_uint
    }
}
