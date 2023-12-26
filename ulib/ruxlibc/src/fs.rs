/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_char, c_int};

use ruxos_posix_api::{
    sys_fstat, sys_getcwd, sys_lseek, sys_lstat, sys_mkdir, sys_open, sys_rename, sys_rmdir,
    sys_stat, sys_unlink,
};

use crate::{ctypes, utils::e};

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
#[no_mangle]
pub unsafe extern "C" fn ax_open(
    filename: *const c_char,
    flags: c_int,
    mode: ctypes::mode_t,
) -> c_int {
    e(sys_open(filename, flags, mode))
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
#[no_mangle]
pub unsafe extern "C" fn lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    e(sys_lseek(fd, offset, whence) as _) as _
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn stat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    e(sys_stat(path, buf as _))
}

/// Get file metadata by `fd` and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn fstat(fd: c_int, buf: *mut ctypes::stat) -> c_int {
    e(sys_fstat(fd, buf as *mut core::ffi::c_void))
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
#[no_mangle]
pub unsafe extern "C" fn lstat(path: *const c_char, buf: *mut ctypes::stat) -> c_int {
    e(sys_lstat(path, buf) as _)
}

/// Get the path of the current directory.
#[no_mangle]
pub unsafe extern "C" fn getcwd(buf: *mut c_char, size: usize) -> *mut c_char {
    if buf.is_null() && size != 0 {
        crate::errno::set_errno(axerrno::LinuxError::EINVAL as _);
        return core::ptr::null_mut() as *mut c_char;
    }
    let e = sys_getcwd(buf, size);
    if e < 0 {
        return core::ptr::null_mut() as *mut c_char;
    }
    if e == 0 || buf.read() != '/' as _ {
        crate::errno::set_errno(axerrno::LinuxError::ENOENT as _);
        return core::ptr::null_mut() as *mut c_char;
    }
    buf
}

/// Rename `old` to `new`
/// If new exists, it is first removed.
///
/// Return 0 if the operation succeeds, otherwise return -1.
#[no_mangle]
pub unsafe extern "C" fn rename(old: *const c_char, new: *const c_char) -> c_int {
    e(sys_rename(old, new))
}

/// Remove a directory, which must be empty
///
/// Return 0 if the operation succeeds, otherwise return -1.
#[no_mangle]
pub unsafe extern "C" fn rmdir(pathname: *const c_char) -> c_int {
    e(sys_rmdir(pathname))
}

/// Removes a file from the filesystem.
#[no_mangle]
pub unsafe extern "C" fn unlink(pathname: *const c_char) -> c_int {
    e(sys_unlink(pathname))
}

/// Creates a new directory
#[no_mangle]
pub unsafe extern "C" fn mkdir(pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    e(sys_mkdir(pathname, mode))
}
