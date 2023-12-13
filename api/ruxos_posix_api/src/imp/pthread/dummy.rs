/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use core::ffi::{c_int, c_void};

/// Allocate a specific key for a process shared by all threads.
pub unsafe fn sys_pthread_key_create(
    key: *mut ctypes::pthread_key_t,
    _destr_function: Option<unsafe extern "C" fn(*mut c_void)>,
) -> c_int {
    debug!("sys_pthread_key_create <= {:#x}", key as usize);
    syscall_body!(sys_pthread_key_create, Ok(0))
}

/// Destroy a specific key for a process.
pub fn sys_pthread_key_delete(key: ctypes::pthread_key_t) -> c_int {
    debug!("sys_pthread_key_delete <= {}", key);
    syscall_body!(sys_pthread_key_delete, Ok(0))
}

/// Set the value of a specific key for a thread.
pub fn sys_pthread_setspecific(key: ctypes::pthread_key_t, value: *const c_void) -> c_int {
    debug!("sys_pthread_setspecific <= {}, {:#x}", key, value as usize);
    syscall_body!(sys_pthread_setspecific, Ok(0))
}

/// Get the value of a specific key for a thread.
pub fn sys_pthread_getspecific(key: ctypes::pthread_key_t) -> *mut c_void {
    debug!("sys_pthread_getspecific <= {}", key);
    syscall_body!(
        sys_pthread_getspecific,
        Ok::<*mut c_void, axerrno::LinuxError>(core::ptr::null_mut() as _)
    )
}
