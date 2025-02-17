/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes::{self, gid_t, pid_t, uid_t};
use core::ffi::c_int;

/// Set file mode creation mask
///
/// TODO:
pub fn sys_umask(mode: ctypes::mode_t) -> ctypes::mode_t {
    debug!("sys_umask <= mode: {:x}", mode);
    syscall_body!(sys_umask, Ok(0))
}

/// Returns the effective user ID of the calling process
pub fn sys_geteuid() -> core::ffi::c_uint {
    syscall_body!(sys_geteuid, Ok(1000))
}

/// Returns the effective groupe ID of the calling process
pub fn sys_getegid() -> core::ffi::c_uint {
    syscall_body!(sys_getegid, Ok(1000))
}

/// Get current real user ID.
pub fn sys_getuid() -> c_int {
    syscall_body!(sys_getuid, Ok(1000))
}

/// Get current real group ID.
pub fn sys_getgid() -> c_int {
    syscall_body!(sys_getgid, Ok(1000))
}

/// set current user id
pub fn sys_setuid(uid: uid_t) -> c_int {
    debug!("sys_setuid: uid {}", uid);
    syscall_body!(sys_setuid, Ok(0))
}

/// set current group id
pub fn sys_setgid(gid: gid_t) -> c_int {
    debug!("sys_setgid: gid {}", gid);
    syscall_body!(sys_setgid, Ok(0))
}

/// get process gid
pub fn sys_getpgid(pid: pid_t) -> c_int {
    debug!("sys_getpgid: getting pgid of pid {} ", pid);
    syscall_body!(sys_getpgid, Ok(1000))
}

/// set process gid
pub fn sys_setpgid(pid: pid_t, pgid: pid_t) -> c_int {
    debug!("sys_setpgid: pid {}, pgid {} ", pid, pgid);
    syscall_body!(sys_setpgid, Ok(0))
}

/// set process sid (empty implementation)
///
/// TODO:
pub fn sys_setsid() -> c_int {
    warn!("sys_setsid: do nothing",);
    syscall_body!(sys_setsid, Ok(0))
}
