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
use axerrno::LinuxError;
use axfs_vfs::VfsNodeAttr;
use bitflags::bitflags;
use bitmaps::Bitmap;
use capability::Cap;
use core::marker::Send;
use core::marker::Sync;
use flatten_objects::FlattenObjects;

use axerrno::LinuxResult;
use axfs_vfs::AbsPath;
use axio::PollState;

/// Maximum number of files per process
pub const RUX_FILE_LIMIT: usize = 1024;

/// A table of file descriptors, containing a collection of file objects and their associated flags(CLOEXEC).
pub struct FdTable {
    /// A collection of file objects, indexed by their file descriptor numbers.
    files: FlattenObjects<Arc<dyn FileLike>, RUX_FILE_LIMIT>,
    /// A bitmap for tracking `FD_CLOEXEC` flags for each file descriptor.
    /// If a bit is set, the corresponding file descriptor has the `FD_CLOEXEC` flag enabled.
    cloexec_bitmap: Bitmap<RUX_FILE_LIMIT>,
}

impl Clone for FdTable {
    fn clone(&self) -> Self {
        // get all file descriptors from the original file system to copy them to the new one
        // TODO: make this more efficient by only copying the used file descriptors
        let mut new_files = FlattenObjects::new();
        for fd in 0..self.files.capacity() {
            if let Some(f) = self.files.get(fd) {
                new_files.add_at(fd, f.clone()).unwrap();
            }
        }
        Self {
            files: new_files,
            cloexec_bitmap: self.cloexec_bitmap,
        }
    }
}

impl Default for FdTable {
    fn default() -> Self {
        FdTable {
            files: FlattenObjects::new(),
            cloexec_bitmap: Bitmap::new(),
        }
    }
}

impl FdTable {
    /// Retrieves the file object associated with the given file descriptor (fd).
    ///
    /// Returns `Some` with the file object if the file descriptor exists, or `None` if not.
    pub fn get(&self, fd: usize) -> Option<&Arc<dyn FileLike>> {
        self.files.get(fd)
    }

    /// Adds a new file object to the table and associates it with a file descriptor.
    ///
    /// Also sets the `FD_CLOEXEC` flag for the file descriptor based on the `flags` argument.
    /// Returns the assigned file descriptor number (`fd`) if successful, or `None` if the table is full.
    pub fn add(&mut self, file: Arc<dyn FileLike>, flags: OpenFlags) -> Option<usize> {
        if let Some(fd) = self.files.add(file) {
            debug_assert!(!self.cloexec_bitmap.get(fd));
            if flags.contains(OpenFlags::O_CLOEXEC) {
                self.cloexec_bitmap.set(fd, true);
            }
            Some(fd)
        } else {
            None
        }
    }

    /// Adds a file object to the table at a specific file descriptor.
    /// It won't be add if the specified fd in the fdtable already exists
    pub fn add_at(&mut self, fd: usize, file: Arc<dyn FileLike>) -> Option<usize> {
        self.files.add_at(fd, file)
    }

    /// Retrieves the `FD_CLOEXEC` flag for the specified file descriptor.
    ///
    /// Returns `true` if the flag is set, otherwise `false`.
    pub fn get_cloexec(&self, fd: usize) -> bool {
        self.cloexec_bitmap.get(fd)
    }

    /// Sets the `FD_CLOEXEC` flag for the specified file descriptor.
    pub fn set_cloexec(&mut self, fd: usize, cloexec: bool) {
        self.cloexec_bitmap.set(fd, cloexec);
    }

    /// Removes a file descriptor from the table.
    ///
    /// This will clear the `FD_CLOEXEC` flag for the file descriptor and remove the file object.
    pub fn remove(&mut self, fd: usize) -> Option<Arc<dyn FileLike>> {
        self.cloexec_bitmap.set(fd, false);
        // use map_or because RAII. the Arc should be released here. You should not use the return Arc
        self.files.remove(fd)
    }

    /// Closes all file descriptors with the `FD_CLOEXEC` flag set.
    ///
    /// This will remove all file descriptors marked for close-on-exec from the table.
    pub fn do_close_on_exec(&mut self) {
        for fd in self.cloexec_bitmap.into_iter() {
            self.files.remove(fd);
        }
        self.cloexec_bitmap = Bitmap::new()
    }

    /// Duplicates a file descriptor and returns a new file descriptor.
    ///
    /// The two file descriptors do not share file descriptor flags (the close-on-exec flag).
    /// The close-on-exec flag (FD_CLOEXEC; see fcntl(2)) for the duplicate descriptor is off.
    pub fn dup(&mut self, fd: usize) -> LinuxResult<usize> {
        let f = self.files.get(fd).ok_or(LinuxError::EBADF)?.clone();
        let new_fd = self.files.add(f).ok_or(LinuxError::EMFILE)?;
        debug_assert!(!self.cloexec_bitmap.get(new_fd));
        Ok(new_fd)
    }

    /// Duplicates a file descriptor to a specific file descriptor number, replacing it if necessary.
    ///
    /// If the file descriptor `newfd` was previously open, it is silently closed before being reused.
    pub fn dup3(&mut self, old_fd: usize, new_fd: usize, cloexec: bool) -> LinuxResult<usize> {
        let f = self.files.get(old_fd).ok_or(LinuxError::EBADF)?.clone();
        self.files.remove(new_fd);
        self.files.add_at(new_fd, f);
        self.cloexec_bitmap.set(new_fd, cloexec);
        Ok(new_fd)
    }

    /// Duplicate the file descriptor fd using the lowest-numbered available file descriptor greater than or equal to `bound`.
    pub fn dup_with_low_bound(
        &mut self,
        fd: usize,
        bound: usize,
        cloexec: bool,
    ) -> LinuxResult<usize> {
        let f = self.files.get(fd).ok_or(LinuxError::EBADF)?.clone();
        let new_fd = self
            .files
            .add_with_low_bound(f, bound)
            .ok_or(LinuxError::EMFILE)?;
        debug_assert!(!self.cloexec_bitmap.get(new_fd));
        self.cloexec_bitmap.set(new_fd, cloexec);
        Ok(new_fd)
    }

    /// Closes all file objects in the file descriptor table.
    pub fn close_all_files(&mut self) {
        for fd in 0..self.files.capacity() {
            if self.files.get(fd).is_some() {
                self.files.remove(fd).unwrap();
            }
        }
        // this code might not be necessary
        self.cloexec_bitmap = Bitmap::new();
    }
}

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
#[derive(Default)]
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

#[cfg(target_arch = "aarch64")]
impl From<VfsNodeAttr> for RuxStat {
    fn from(attr: VfsNodeAttr) -> Self {
        Self {
            st_dev: 0,
            st_ino: attr.ino(),
            st_nlink: 1,
            st_mode: ((attr.file_type() as u32) << 12) | attr.perm().bits() as u32,
            st_uid: 1000,
            st_gid: 1000,
            st_rdev: 0,
            __pad: 0,
            st_size: attr.size() as _,
            st_blksize: 512,
            __pad2: 0,
            st_blocks: attr.blocks() as _,
            st_atime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            __unused: [0; 2],
        }
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "riscv64"))]
impl From<VfsNodeAttr> for RuxStat {
    fn from(attr: VfsNodeAttr) -> Self {
        Self {
            st_dev: 0,
            st_ino: attr.ino(),
            st_nlink: 1,
            st_mode: ((attr.file_type() as u32) << 12) | attr.perm().bits() as u32,
            st_uid: 1000,
            st_gid: 1000,
            __pad0: 0,
            st_rdev: 0,
            st_size: attr.size() as _,
            st_blksize: 512,
            st_blocks: attr.blocks() as _,
            st_atime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_mtime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            st_ctime: RuxTimeSpec {
                tv_sec: 0,
                tv_nsec: 0,
            },
            __unused: [0; 3],
        }
    }
}

/// Trait for file-like objects in a file descriptor table.
pub trait FileLike: Send + Sync {
    /// Get the absolute path of the file-like object.
    fn path(&self) -> AbsPath;

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

    /// Sets the flags such as `nonblocking` for the file-like object.
    /// Only File Status Flags can be changed once a file is opened. Other flags will be ignored.
    fn set_flags(&self, _flags: OpenFlags) -> LinuxResult;

    /// Return File Access Modes and File Status Flags. Creation Flags needn't store.
    /// `sys_fcntl` command `F_GETFL` will need both kinds
    fn flags(&self) -> OpenFlags;

    /// Handles ioctl commands for the device.
    fn ioctl(&self, _cmd: usize, _arg: usize) -> LinuxResult<usize> {
        Err(LinuxError::ENOTTY)
    }
}

bitflags! {
    /// Bit flags for file opening and creation options
    ///
    /// These flags control file access modes, creation semantics, and status flags.
    /// The `file creation flags` affect the semantics of the open operation itself, while the `file status
    /// flag` affect the semantics of subsequent I/O operations.
    /// The file status flags can be retrieved and (in some cases) modified in `sys_fcntl`.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct OpenFlags: i32 {
        // ----------------------------
        // File Access Modes (won't be changed once a file is opened)
        // Note: `bitflags` do not encourage zero bit flag. So `O_RDONLY`` is represented as 0 (no bits set)
        // ----------------------------
        /// Write-only access
        const O_WRONLY      = 1;
        /// Read-write access
        const O_RDWR        = 2;
        /// Mask for access mode bits (O_RDONLY, O_WRONLY, O_RDWR)
        const O_ACCMODE     = 3;

        // ----------------------------
        // Creation Flags
        // ----------------------------
        /// Close-on-exec flag
        const O_CLOEXEC     = 0o2000000;
        /// Create file if it doesn't exist
        const O_CREAT       = 0o100;
        /// Fail if O_CREAT and file exists
        const O_EXCL        = 0o200;
        /// Fail if path is not a directory
        const O_DIRECTORY   = 0o200000;
        /// Don't assign controlling terminal
        const O_NOCTTY      = 0o400;
        /// Don't follow symbolic links
        const O_NOFOLLOW    = 0o400000;
        /// Truncate existing file to zero length
        const O_TRUNC       = 0o1000;
        /// Create unnamed temporary file
        /// (requires O_DIRECTORY if used with directory)
        const O_TMPFILE     = 0o20200000;

        // ----------------------------
        // Status flags (Will be stored in `FileLike` dyn obj using fn `flags()`)
        // ----------------------------
        /// Non-blocking mode
        const O_NONBLOCK    = 0o4000;
        /// Data synchronization (synchronous I/O)
        const O_DSYNC       = 0o10000;
        /// File synchronization (sync I/O file integrity)
        const O_SYNC        = 0o4010000;
        /// Synchronize read operations
        const O_RSYNC       = 0o4010000;
        /// Append mode (writes at end of file)
        const O_APPEND      = 0o2000;
        /// Asynchronous I/O notification
        const O_ASYNC       = 0o20000;
        /// Direct I/O (no kernel buffering)
        const O_DIRECT      = 0o40000;
        /// Allow large file support
        const O_LARGEFILE   = 0o100000;
        /// Don't update file access time
        const O_NOATIME     = 0o1000000;
        /// Open path without accessing filesystem. Also named `O_SEARCH`
        const O_PATH        = 0o10000000;
    }
}

impl OpenFlags {
    /// `create_new` flag in `OpenOptions`
    pub const CREATE_NEW: Self = Self::O_CREAT.union(Self::O_EXCL);
    /// Read only flag
    pub const O_RDONLY: Self = Self::empty();
    /// creation flags mask
    pub const CREATION_FLAGS: Self = Self::O_CLOEXEC
        .union(Self::O_CREAT)
        .union(Self::O_DIRECTORY)
        .union(Self::O_EXCL)
        .union(Self::O_NOCTTY)
        .union(Self::O_NOFOLLOW)
        .union(Self::O_TMPFILE)
        .union(Self::O_TRUNC);

    /// Checks if the file is opened in a readable mode
    /// (O_RDONLY or O_RDWR, and not write-only)
    pub fn readable(&self) -> bool {
        !self.contains(Self::O_WRONLY) || self.contains(Self::O_RDWR)
    }

    /// Checks if the file is opened in a writable mode
    /// (O_WRONLY or O_RDWR)
    pub fn writable(&self) -> bool {
        self.contains(Self::O_WRONLY) || self.contains(Self::O_RDWR)
    }

    /// Only return file status flags
    pub fn status_flags(&self) -> Self {
        *self & !Self::O_ACCMODE & !Self::CREATION_FLAGS
    }

    /// Return the file access mode and the file status flags. Used in `sys_fcntl` with `F_GETFL` mode
    pub fn getfl(&self) -> Self {
        *self & !Self::CREATION_FLAGS
    }
}

impl From<OpenFlags> for Cap {
    fn from(openflags: OpenFlags) -> Self {
        let mut cap = Cap::empty();
        if openflags.readable() {
            cap |= Cap::READ;
        }
        if openflags.writable() {
            cap |= Cap::WRITE;
        }
        cap
    }
}
