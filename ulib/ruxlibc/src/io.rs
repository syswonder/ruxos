/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_int, c_void};
#[cfg(feature = "fd")]
use ruxos_posix_api::sys_ioctl;

#[cfg(not(test))]
use ruxos_posix_api::sys_write;
use ruxos_posix_api::{sys_read, sys_writev};

use crate::{ctypes, utils::e};

/// Read data from the file indicated by `fd`.
///
/// Return the read size if success.
#[no_mangle]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: usize) -> ctypes::ssize_t {
    e(sys_read(fd, buf, count) as _) as _
}

/// Write data to the file indicated by `fd`.
///
/// Return the written size if success.
#[no_mangle]
#[cfg(not(test))]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: usize) -> ctypes::ssize_t {
    e(sys_write(fd, buf, count) as _) as _
}

/// Write a vector.
#[no_mangle]
pub unsafe extern "C" fn writev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
) -> ctypes::ssize_t {
    e(sys_writev(fd, iov, iocnt) as _) as _
}

/// Manipulate file descriptor.
///
/// TODO: `SET/GET` command is ignored
#[cfg(feature = "fd")]
#[no_mangle]
pub unsafe extern "C" fn rux_ioctl(fd: c_int, req: c_int, arg: usize) -> c_int {
    e(sys_ioctl(fd, req.try_into().unwrap(), arg))
}
