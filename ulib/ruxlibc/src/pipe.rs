/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;

use ruxos_posix_api::sys_pipe;

use crate::utils::e;

/// Create a pipe
///
/// Return 0 if succeed
#[no_mangle]
pub unsafe extern "C" fn pipe(fd: *mut c_int) -> c_int {
    let fds = unsafe { core::slice::from_raw_parts_mut(fd, 2) };
    e(sys_pipe(fds))
}
