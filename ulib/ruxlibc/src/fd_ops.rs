/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::{ctypes, utils::e};
use axerrno::LinuxError;
use core::ffi::c_int;
use ruxos_posix_api::{sys_close, sys_dup, sys_dup2, sys_fcntl};

/// Close a file by `fd`.
#[no_mangle]
pub unsafe extern "C" fn close(fd: c_int) -> c_int {
    e(sys_close(fd))
}

/// Duplicate a file descriptor.
#[no_mangle]
pub unsafe extern "C" fn dup(old_fd: c_int) -> c_int {
    e(sys_dup(old_fd))
}

/// Duplicate a file descriptor, use file descriptor specified in `new_fd`.
#[no_mangle]
pub unsafe extern "C" fn dup2(old_fd: c_int, new_fd: c_int) -> c_int {
    e(sys_dup2(old_fd, new_fd))
}

/// Duplicate a file descriptor, the caller can force the close-on-exec flag to
/// be set for the new file descriptor by specifying `O_CLOEXEC` in flags.
///
/// If oldfd equals newfd, then `dup3()` fails with the error `EINVAL`.
#[no_mangle]
pub unsafe extern "C" fn dup3(old_fd: c_int, new_fd: c_int, flags: c_int) -> c_int {
    if old_fd == new_fd {
        return e((LinuxError::EINVAL as c_int).wrapping_neg());
    }
    let r = e(sys_dup2(old_fd, new_fd));
    if r < 0 {
        r
    } else {
        if flags as u32 & ctypes::O_CLOEXEC != 0 {
            e(sys_fcntl(
                new_fd,
                ctypes::F_SETFD as c_int,
                ctypes::FD_CLOEXEC as usize,
            ));
        }
        new_fd
    }
}

/// Manipulate file descriptor.
///
/// TODO: `SET/GET` command is ignored
#[no_mangle]
pub unsafe extern "C" fn ax_fcntl(fd: c_int, cmd: c_int, arg: usize) -> c_int {
    e(sys_fcntl(fd, cmd, arg))
}
