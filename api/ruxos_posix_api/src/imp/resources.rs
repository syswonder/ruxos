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
use core::ffi::c_int;

/// Get resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_getrlimit(resource: c_int, rlimits: *mut ctypes::rlimit) -> c_int {
    debug!("sys_getrlimit <= {} {:#x}", resource, rlimits as usize);
    syscall_body!(sys_getrlimit, {
        match resource as u32 {
            ctypes::RLIMIT_CPU => {}
            ctypes::RLIMIT_FSIZE => {}
            ctypes::RLIMIT_DATA => {}
            ctypes::RLIMIT_STACK => {}
            ctypes::RLIMIT_CORE => {}
            ctypes::RLIMIT_RSS => {}
            ctypes::RLIMIT_NPROC => {}
            ctypes::RLIMIT_NOFILE => {}
            ctypes::RLIMIT_MEMLOCK => {}
            ctypes::RLIMIT_AS => {}
            ctypes::RLIMIT_LOCKS => {}
            ctypes::RLIMIT_SIGPENDING => {}
            ctypes::RLIMIT_MSGQUEUE => {}
            ctypes::RLIMIT_NICE => {}
            ctypes::RLIMIT_RTPRIO => {}
            ctypes::RLIMIT_RTTIME => {}
            ctypes::RLIMIT_NLIMITS => {}
            _ => return Err(LinuxError::EINVAL),
        }
        if rlimits.is_null() {
            return Ok(0);
        }
        match resource as u32 {
            ctypes::RLIMIT_CPU => {}
            ctypes::RLIMIT_FSIZE => {}
            ctypes::RLIMIT_DATA => {}
            ctypes::RLIMIT_STACK => unsafe {
                (*rlimits).rlim_cur = ruxconfig::TASK_STACK_SIZE as _;
                (*rlimits).rlim_max = ruxconfig::TASK_STACK_SIZE as _;
            },
            ctypes::RLIMIT_CORE => {}
            ctypes::RLIMIT_RSS => {}
            ctypes::RLIMIT_NPROC => unsafe {
                (*rlimits).rlim_cur = 1;
                (*rlimits).rlim_max = 1;
            },
            #[cfg(feature = "fd")]
            ctypes::RLIMIT_NOFILE => unsafe {
                (*rlimits).rlim_cur = ruxtask::fs::RUX_FILE_LIMIT as _;
                (*rlimits).rlim_max = ruxtask::fs::RUX_FILE_LIMIT as _;
            },
            ctypes::RLIMIT_MEMLOCK => {}
            ctypes::RLIMIT_AS => {}
            ctypes::RLIMIT_LOCKS => {}
            ctypes::RLIMIT_SIGPENDING => {}
            ctypes::RLIMIT_MSGQUEUE => {}
            ctypes::RLIMIT_NICE => {}
            ctypes::RLIMIT_RTPRIO => {}
            ctypes::RLIMIT_RTTIME => {}
            ctypes::RLIMIT_NLIMITS => {}
            _ => {}
        }
        Ok(0)
    })
}

/// Set resource limitations
///
/// TODO: support more resource types
pub unsafe fn sys_setrlimit(resource: c_int, rlimits: *const ctypes::rlimit) -> c_int {
    debug!("sys_setrlimit <= {} {:#x}", resource, rlimits as usize);
    syscall_body!(sys_setrlimit, {
        match resource as u32 {
            ctypes::RLIMIT_DATA => {}
            ctypes::RLIMIT_STACK => {}
            ctypes::RLIMIT_NOFILE => {}
            _ => return Err(LinuxError::EINVAL),
        }
        // Currently do not support set resources
        Ok(0)
    })
}

/// set/get resource limitations
pub unsafe fn sys_prlimit64(
    _pid: ctypes::pid_t,
    resource: c_int,
    new_limit: *const ctypes::rlimit,
    old_limit: *mut ctypes::rlimit,
) -> c_int {
    debug!("sys_prlimit64 <= resource: {}", resource);
    if !new_limit.is_null() {
        return sys_setrlimit(resource, new_limit);
    }
    if !old_limit.is_null() {
        return sys_getrlimit(resource, old_limit);
    }
    0
}
