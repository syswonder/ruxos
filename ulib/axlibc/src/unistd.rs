/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use arceos_posix_api::{sys_exit, sys_getpid};
use core::ffi::c_int;

/// Get current thread ID.
#[no_mangle]
pub unsafe extern "C" fn getpid() -> c_int {
    sys_getpid()
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
