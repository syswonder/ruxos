/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use arceos_posix_api::{sys_clock_gettime, sys_clock_settime, sys_nanosleep};
use core::ffi::c_int;

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
