/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! fd table and FileLike trait for file system
#![no_std]
extern crate alloc;
use alloc::sync::Arc;
use core::marker::Send;
use core::marker::Sync;

use axerrno::LinuxResult;
use axio::PollState;

#[derive(Default)]
///Rust version for struct timespec in ctypes. Represents a high-resolution time specification.
pub struct RuxTimeSpec {
    /// Whole seconds part of the timespec.
    pub tv_sec: core::ffi::c_longlong,
    /// Nanoseconds part of the timespec, complementing `tv_sec`.
    pub tv_nsec: core::ffi::c_long,
}

///Rust version for struct stat in ctypes. Represents file status information.
#[cfg(target_arch = "aarch64")]
#[derive(Default)]
pub struct RuxStat {
    /// Device identifier.
    pub st_dev: u64,
    /// Inode number.
    pub st_ino: u64,
    /// File mode and permissions.
    pub st_mode: core::ffi::c_uint,
    /// Number of hard links.
    pub st_nlink: u32,
    /// User ID of owner.
    pub st_uid: core::ffi::c_uint,
    /// Group ID of owner.
    pub st_gid: core::ffi::c_uint,
    /// Device ID (if special file).
    pub st_rdev: u64,
    /// Padding to maintain alignment.
    pub __pad: core::ffi::c_ulong,
    /// Total size, in bytes.
    pub st_size: i64,
    /// Block size for filesystem I/O.
    pub st_blksize: core::ffi::c_long,
    /// Padding to maintain alignment.
    pub __pad2: core::ffi::c_int,
    /// Number of 512B blocks allocated.
    pub st_blocks: i64,
    /// Time of last access.
    pub st_atime: RuxTimeSpec,
    /// Time of last modification.
    pub st_mtime: RuxTimeSpec,
    /// Time of last status change.
    pub st_ctime: RuxTimeSpec,
    /// Unused space, reserved for future use.
    pub __unused: [core::ffi::c_uint; 2usize],
}
///Rust version for struct stat in ctypes. Represents file status information.
#[cfg(any(target_arch = "x86_64", target_arch = "riscv64"))]
pub struct RuxStat {
    /// Device identifier.
    pub st_dev: u64,
    /// Inode number.
    pub st_ino: u64,
    /// Number of hard links.
    pub st_nlink: u64,
    /// File mode and permissions.
    pub st_mode: core::ffi::c_uint,
    /// User ID of owner.
    pub st_uid: core::ffi::c_uint,
    /// Group ID of owner.
    pub st_gid: core::ffi::c_uint,
    /// Padding to maintain alignment.
    pub __pad0: core::ffi::c_uint,
    /// Device ID (if special file).
    pub st_rdev: u64,
    /// Total size, in bytes.
    pub st_size: i64,
    /// Block size for filesystem I/O.
    pub st_blksize: core::ffi::c_long,
    /// Number of 512B blocks allocated.
    pub st_blocks: i64,
    /// Time of last access.
    pub st_atime: RuxTimeSpec,
    /// Time of last modification.
    pub st_mtime: RuxTimeSpec,
    /// Time of last status change.
    pub st_ctime: RuxTimeSpec,
    /// Unused space, reserved for future use.
    pub __unused: [core::ffi::c_long; 3usize],
}

/// Trait for file-like objects in a file descriptor table.
pub trait FileLike: Send + Sync {
    /// Reads data from the file-like object into the provided buffer.
    ///
    /// Returns the number of bytes read on success.
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize>;

    /// Writes data from the provided buffer to the file-like object.
    ///
    /// Returns the number of bytes written on success.
    fn write(&self, buf: &[u8]) -> LinuxResult<usize>;

    /// Flushes any buffered data to the file-like object.
    fn flush(&self) -> LinuxResult;

    /// Retrieves metadata about the file-like object.
    fn stat(&self) -> LinuxResult<RuxStat>;

    /// Converts this object into a generic `Any` type, enabling downcasting.
    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync>;

    /// Polls the file-like object for readiness events.
    fn poll(&self) -> LinuxResult<PollState>;

    /// Sets or clears the non-blocking I/O mode for the file-like object.
    fn set_nonblocking(&self, nonblocking: bool) -> LinuxResult;
}
