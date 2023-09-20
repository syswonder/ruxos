/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::{ctypes, utils::e};
use arceos_posix_api as api;
use core::ffi::{c_int, c_void};

/// Returns the `pthread` struct of current thread.
#[no_mangle]
pub unsafe extern "C" fn pthread_self() -> ctypes::pthread_t {
    api::sys_pthread_self()
}

/// Create a new thread with the given entry point and argument.
///
/// If successful, it stores the pointer to the newly created `struct __pthread`
/// in `res` and returns 0.
#[no_mangle]
pub unsafe extern "C" fn pthread_create(
    res: *mut ctypes::pthread_t,
    attr: *const ctypes::pthread_attr_t,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    e(api::sys_pthread_create(res, attr, start_routine, arg))
}

/// Exits the current thread. The value `retval` will be returned to the joiner.
#[no_mangle]
pub unsafe extern "C" fn pthread_exit(retval: *mut c_void) -> ! {
    api::sys_pthread_exit(retval)
}

/// Waits for the given thread to exit, and stores the return value in `retval`.
#[no_mangle]
pub unsafe extern "C" fn pthread_join(
    thread: ctypes::pthread_t,
    retval: *mut *mut c_void,
) -> c_int {
    e(api::sys_pthread_join(thread, retval))
}

/// Initialize a mutex.
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_init(
    mutex: *mut ctypes::pthread_mutex_t,
    attr: *const ctypes::pthread_mutexattr_t,
) -> c_int {
    e(api::sys_pthread_mutex_init(mutex, attr))
}

/// Lock the given mutex.
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_lock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    e(api::sys_pthread_mutex_lock(mutex))
}

/// Lock the given mutex. If the mutex is already locked, it returns immediatly with the error
/// code EBUSY.
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_trylock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    e(api::sys_pthread_mutex_trylock(mutex))
}

/// Unlock the given mutex.
#[no_mangle]
pub unsafe extern "C" fn pthread_mutex_unlock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    e(api::sys_pthread_mutex_unlock(mutex))
}

/// Initialize a condition variable
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_init(
    condvar: *mut ctypes::pthread_cond_t,
    attr: *mut ctypes::pthread_condattr_t,
) -> c_int {
    e(api::sys_pthread_cond_init(condvar, attr))
}

/// Wait for the condition variable to be signaled
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_wait(
    condvar: *mut ctypes::pthread_cond_t,
    mutex: *mut ctypes::pthread_mutex_t,
) -> c_int {
    e(api::sys_pthread_cond_wait(condvar, mutex))
}

/// Restarts one of the threads that are waiting on the condition variable.
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_signal(condvar: *mut ctypes::pthread_cond_t) -> c_int {
    e(api::sys_pthread_cond_signal(condvar))
}

/// Restarts all the threads that are waiting on the condition variable.
#[no_mangle]
pub unsafe extern "C" fn pthread_cond_broadcast(condvar: *mut ctypes::pthread_cond_t) -> c_int {
    e(api::sys_pthread_cond_broadcast(condvar))
}
