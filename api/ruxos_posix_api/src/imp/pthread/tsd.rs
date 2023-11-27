/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use axerrno::LinuxError;
use core::ffi::{c_int, c_void};
use ruxtask::tsd::DestrFunction;

/// Allocate a specific key for a process shared by all threads.
pub unsafe fn sys_pthread_key_create(
    key: *mut ctypes::pthread_key_t,
    destr_function: Option<DestrFunction>,
) -> c_int {
    debug!("sys_pthread_key_create <= {:#x}", key as usize);
    syscall_body!(sys_pthread_key_create, {
        if let Some(k) = ruxtask::current().alloc_key(destr_function) {
            unsafe {
                *key = k as ctypes::pthread_key_t;
            }
            Ok(0)
        } else {
            Err(LinuxError::EAGAIN)
        }
    })
}

/// Destroy a specific key for a process.
pub fn sys_pthread_key_delete(key: ctypes::pthread_key_t) -> c_int {
    debug!("sys_pthread_key_delete <= {}", key);
    syscall_body!(sys_pthread_key_delete, {
        if let Some(_) = ruxtask::current().free_key(key as usize) {
            Ok(0)
        } else {
            Err(LinuxError::EINVAL)
        }
    })
}

/// Set the value of a specific key for a thread.
pub fn sys_pthread_setspecific(key: ctypes::pthread_key_t, value: *const c_void) -> c_int {
    debug!("sys_pthread_setspecific <= {}, {:#x}", key, value as usize);
    syscall_body!(sys_pthread_setspecific, {
        if let Some(_) = ruxtask::current().set_tsd(key as usize, value as *mut c_void) {
            Ok(0)
        } else {
            Err(LinuxError::EINVAL)
        }
    })
}

/// Get the value of a specific key for a thread.
pub fn sys_pthread_getspecific(key: ctypes::pthread_key_t) -> *mut c_void {
    debug!("sys_pthread_getspecific <= {}", key);
    syscall_body!(sys_pthread_getspecific, {
        if let Some(tsd) = ruxtask::current().get_tsd(key as usize) {
            Ok(tsd)
        } else {
            // return null
            Ok(core::ptr::null_mut())
        }
    })
}
