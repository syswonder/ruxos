/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::{ctypes, utils::check_null_mut_ptr};

use axerrno::{LinuxError, LinuxResult};
use axsync::Mutex;

use core::ffi::c_int;
use core::mem::{size_of, ManuallyDrop};

static_assertions::const_assert_eq!(
    size_of::<PthreadMutex>(),
    size_of::<ctypes::pthread_mutex_t>()
);

#[repr(C)]
pub struct PthreadMutex(Mutex<()>);

impl PthreadMutex {
    const fn new() -> Self {
        Self(Mutex::new(()))
    }

    fn lock(&self) -> LinuxResult {
        let _guard = ManuallyDrop::new(self.0.lock());
        Ok(())
    }

    fn unlock(&self) -> LinuxResult {
        unsafe { self.0.force_unlock() };
        Ok(())
    }

    fn trylock(&self) -> LinuxResult {
        match self.0.try_lock() {
            Some(mutex_guard) => {
                let _guard = ManuallyDrop::new(mutex_guard);
                Ok(())
            }
            None => Err(LinuxError::EBUSY),
        }
    }
}

/// Initialize a mutex.
pub fn sys_pthread_mutex_init(
    mutex: *mut ctypes::pthread_mutex_t,
    _attr: *const ctypes::pthread_mutexattr_t,
) -> c_int {
    debug!("sys_pthread_mutex_init <= {:#x}", mutex as usize);
    syscall_body!(sys_pthread_mutex_init, {
        check_null_mut_ptr(mutex)?;
        unsafe {
            mutex.cast::<PthreadMutex>().write(PthreadMutex::new());
        }
        Ok(0)
    })
}

/// Destroy the given mutex.
pub fn sys_pthread_mutex_destroy(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    debug!("sys_pthread_mutex_destroy <= {:#x}", mutex as usize);
    syscall_body!(sys_pthread_mutex_destroy, {
        check_null_mut_ptr(mutex)?;
        unsafe {
            mutex.cast::<PthreadMutex>().drop_in_place();
        }
        Ok(0)
    })
}

/// Lock the given mutex.
pub fn sys_pthread_mutex_lock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    debug!("sys_pthread_mutex_lock <= {:#x}", mutex as usize);
    syscall_body!(sys_pthread_mutex_lock, {
        check_null_mut_ptr(mutex)?;
        unsafe {
            (*mutex.cast::<PthreadMutex>()).lock()?;
        }
        Ok(0)
    })
}

/// Unlock the given mutex.
pub fn sys_pthread_mutex_unlock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    debug!("sys_pthread_mutex_unlock <= {:#x}", mutex as usize);
    syscall_body!(sys_pthread_mutex_unlock, {
        check_null_mut_ptr(mutex)?;
        unsafe {
            (*mutex.cast::<PthreadMutex>()).unlock()?;
        }
        Ok(0)
    })
}

/// Lock the given mutex like sys_pthread_mutex_lock, except that it does not
/// block the calling thread if the mutex is already locked.
///
/// Instead, it returns with the error code EBUSY.
pub fn sys_pthread_mutex_trylock(mutex: *mut ctypes::pthread_mutex_t) -> c_int {
    debug!("sys_pthread_mutex_trylock <= {:#x}", mutex as usize);
    syscall_body!(sys_pthread_mutex_trylock, {
        check_null_mut_ptr(mutex)?;
        unsafe {
            (*mutex.cast::<PthreadMutex>()).trylock()?;
        }
        Ok(0)
    })
}
