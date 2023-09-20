/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use core::ffi::c_int;

use crate::{ctypes, sys_pthread_mutex_lock, sys_pthread_mutex_unlock};
use axerrno::LinuxResult;
use axtask::WaitQueue;
use core::mem::size_of;

static_assertions::const_assert_eq!(size_of::<Condvar>(), size_of::<ctypes::pthread_cond_t>());

#[repr(C)]
pub struct Condvar {
    wq: WaitQueue,
}

impl Condvar {
    const fn new() -> Self {
        Self {
            wq: WaitQueue::new(),
        }
    }

    fn wait(&self, mutex: *mut ctypes::pthread_mutex_t) -> LinuxResult {
        let ret = sys_pthread_mutex_unlock(mutex);
        if ret < 0 {
            return Err(axerrno::LinuxError::try_from(ret).unwrap());
        }
        self.wq.wait();
        let ret = sys_pthread_mutex_lock(mutex);
        if ret < 0 {
            return Err(axerrno::LinuxError::try_from(ret).unwrap());
        }
        Ok(())
    }

    fn notify_one(&self) -> LinuxResult {
        self.wq.notify_one(true);
        Ok(())
    }

    fn notify_all(&self) -> LinuxResult {
        self.wq.notify_all(true);
        Ok(())
    }
}

/// Initialize a condition variable
pub unsafe fn sys_pthread_cond_init(
    condvar: *mut ctypes::pthread_cond_t,
    _attr: *mut ctypes::pthread_condattr_t,
) -> c_int {
    debug!("sys_pthread_cond_init <= {:#x}", condvar as usize);
    syscall_body!(sys_pthread_cond_init, {
        condvar.cast::<Condvar>().write(Condvar::new());
        Ok(0)
    })
}

/// Wait for the condition variable to be signaled
pub unsafe fn sys_pthread_cond_wait(
    condvar: *mut ctypes::pthread_cond_t,
    mutex: *mut ctypes::pthread_mutex_t,
) -> c_int {
    debug!(
        "sys_pthread_cond_wait <= {:#x}, {:#x}",
        condvar as usize, mutex as usize
    );
    syscall_body!(sys_pthread_cond_wait, {
        (*condvar.cast::<Condvar>()).wait(mutex)?;
        Ok(0)
    })
}

/// Restarts one of the threads that are waiting on the condition variable.
pub unsafe fn sys_pthread_cond_signal(condvar: *mut ctypes::pthread_cond_t) -> c_int {
    debug!("sys_pthread_cond_signal <= {:#x}", condvar as usize);
    syscall_body!(sys_pthread_cond_signal, {
        (*condvar.cast::<Condvar>()).notify_one()?;
        Ok(0)
    })
}

/// Restarts all the threads that are waiting on the condition variable.
pub unsafe fn sys_pthread_cond_broadcast(condvar: *mut ctypes::pthread_cond_t) -> c_int {
    debug!("sys_pthread_cond_broadcast <= {:#x}", condvar as usize);
    syscall_body!(sys_pthread_cond_broadcast, {
        (*condvar.cast::<Condvar>()).notify_all()?;
        Ok(0)
    })
}
