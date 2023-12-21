/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Signal implementation, used by musl

use core::ffi::c_int;

use crate::ctypes;

/// Set mask for given thread
pub fn sys_rt_sigprocmask(
    flag: c_int,
    _new_mask: *const usize,
    _old_mask: *mut usize,
    sigsetsize: usize,
) -> c_int {
    debug!(
        "sys_rt_sigprocmask <= flag: {}, sigsetsize: {}",
        flag, sigsetsize
    );
    syscall_body!(sys_rt_sigprocmask, Ok(0))
}

/// sigaction syscall for A64 musl
pub fn sys_rt_sigaction(
    sig: c_int,
    _sa: *const ctypes::sigaction,
    _old: *mut ctypes::sigaction,
    _sigsetsize: ctypes::size_t,
) -> c_int {
    debug!("sys_rt_sigaction <= sig: {}", sig);
    syscall_body!(sys_rt_sigaction, Ok(0))
}
