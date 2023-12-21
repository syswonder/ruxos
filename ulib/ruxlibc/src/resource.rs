/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;

use ruxos_posix_api::{sys_getrlimit, sys_setrlimit};

use crate::utils::e;

/// Get resource limitations
#[no_mangle]
pub unsafe extern "C" fn getrlimit(resource: c_int, rlimits: *mut crate::ctypes::rlimit) -> c_int {
    e(sys_getrlimit(resource, rlimits))
}

/// Set resource limitations
#[no_mangle]
pub unsafe extern "C" fn setrlimit(
    resource: c_int,
    rlimits: *const crate::ctypes::rlimit,
) -> c_int {
    e(sys_setrlimit(resource, rlimits))
}
