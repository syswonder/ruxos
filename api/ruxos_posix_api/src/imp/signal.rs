/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use core::time::Duration;

use crate::ctypes::k_sigaction;
use crate::ctypes::{self, pid_t};

use axerrno::LinuxError;
use ruxtask::{rx_sigaction, Signal};

/// Set signal handler
pub fn sys_sigaction(
    signum: u8,
    sigaction: Option<&k_sigaction>,
    oldact: Option<&mut k_sigaction>,
) -> c_int {
    debug!("sys_sigaction <= signum: {}", signum,);
    syscall_body!(sys_sigaction, {
        Signal::sigaction(
            signum,
            sigaction.map(|act| act as *const k_sigaction as *const rx_sigaction),
            oldact.map(|old| old as *mut k_sigaction as *mut rx_sigaction),
        );
        Ok(0)
    })
}

/// Set a timer to send a signal to the current process after a specified time
pub unsafe fn sys_setitimer(which: c_int, new: *const ctypes::itimerval) -> c_int {
    debug!("sys_setitimer <= which: {}, new: {:p}", which, new);
    syscall_body!(sys_setitimer, {
        let which = which as usize;
        let new_interval = Duration::from((*new).it_interval).as_nanos() as u64;
        Signal::timer_interval(which, Some(new_interval));

        let new_ddl =
            ruxhal::time::current_time_nanos() + Duration::from((*new).it_value).as_nanos() as u64;
        Signal::timer_deadline(which, Some(new_ddl));
        Ok(0)
    })
}

/// Get timer to send signal after some time
pub unsafe fn sys_getitimer(which: c_int, curr_value: *mut ctypes::itimerval) -> c_int {
    debug!(
        "sys_getitimer <= which: {}, curr_value: {:p}",
        which, curr_value
    );
    syscall_body!(sys_getitimer, {
        let ddl = Duration::from_nanos(Signal::timer_deadline(which as usize, None).unwrap());
        if ddl.as_nanos() == 0 {
            return Err(LinuxError::EINVAL);
        }
        let mut now: ctypes::timespec = ctypes::timespec::default();
        unsafe {
            crate::sys_clock_gettime(0, &mut now);
        }
        let now = Duration::from(now);
        if ddl > now {
            (*curr_value).it_value = ctypes::timeval::from(ddl - now);
        } else {
            (*curr_value).it_value = ctypes::timeval::from(Duration::new(0, 0));
        }
        (*curr_value).it_interval =
            Duration::from_nanos(Signal::timer_interval(which as usize, None).unwrap()).into();
        Ok(0)
    })
}

/// Sigal stack
///
/// TODO: implement this && the parameter type should be ctypes::stack_t
pub unsafe fn sys_sigaltstack(
    _ss: *const core::ffi::c_void,
    _old_ss: *mut core::ffi::c_void,
) -> c_int {
    debug!("sys_sigaltstack <= ss: {:p}, old_ss: {:p}", _ss, _old_ss);
    syscall_body!(sys_sigaltstack, Ok(0))
}

/// send a signal to a process
pub fn sys_kill(pid: pid_t, sig: c_int) -> c_int {
    debug!("sys_kill <= pid {} sig {}", pid, sig);
    syscall_body!(sys_kill, {
        match Signal::signal(sig as _, true) {
            None => Err(LinuxError::EINVAL),
            Some(_) => Ok(0),
        }
    })
}

/// send a signal to a thread
/// TODO: send to the specified thread.
pub fn sys_tkill(tid: pid_t, sig: c_int) -> c_int {
    debug!("sys_tkill <= tid {} sig {}", tid, sig);
    sys_kill(tid, sig)
}
