/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;

/// Set file mode creation mask
///
/// TODO:
pub fn sys_umask(mode: ctypes::mode_t) -> ctypes::mode_t {
    debug!("sys_umask <= mode: {:x}", mode);
    syscall_body!(sys_umask, Ok(0))
}

// /// Returns the effective user ID of the calling process
// pub fn sys_geteuid() -> core::ffi::c_uint {
//     syscall_body!(sys_geteuid, Ok(0))
// }
