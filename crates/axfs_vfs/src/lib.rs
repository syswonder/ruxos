/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Virtual filesystem interfaces used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! A filesystem is a set of files and directories (symbol links are not
//! supported currently), collectively referred to as **nodes**, which are
//! conceptually similar to [inodes] in Linux. A file system needs to implement
//! the [`VfsOps`] trait, its files and directories need to implement the
//! [`VfsNodeOps`] trait.
//!
//! The [`VfsOps`] trait provides the following operations on a filesystem:
//!
//! - [`mount()`](VfsOps::mount): Do something when the filesystem is mounted.
//! - [`umount()`](VfsOps::umount): Do something when the filesystem is unmounted.
//! - [`format()`](VfsOps::format): Format the filesystem.
//! - [`statfs()`](VfsOps::statfs): Get the attributes of the filesystem.
//! - [`root_dir()`](VfsOps::root_dir): Get root directory of the filesystem.
//!
//! The [`VfsNodeOps`] trait provides the following operations on a file or a
//! directory:
//!
//! | Operation | Description | file/directory |
//! | --- | --- | --- |
//! | [`open()`](VfsNodeOps::open) | Do something when the node is opened | both |
//! | [`release()`](VfsNodeOps::release) | Do something when the node is closed | both |
//! | [`get_attr()`](VfsNodeOps::get_attr) | Get the attributes of the node | both |
//! | [`read_at()`](VfsNodeOps::read_at) | Read data from the file | file |
//! | [`write_at()`](VfsNodeOps::write_at) | Write data to the file | file |
//! | [`fsync()`](VfsNodeOps::fsync) | Synchronize the file data to disk | file |
//! | [`truncate()`](VfsNodeOps::truncate) | Truncate the file | file |
//! | [`parent()`](VfsNodeOps::parent) | Get the parent directory | directory |
//! | [`lookup()`](VfsNodeOps::lookup) | Lookup the node with the given path | directory |
//! | [`create()`](VfsNodeOps::create) | Create a new node with the given path | directory |
//! | [`link()`](VfsNodeOps::link) | Create a hard link with the given path | directory |
//! | [`unlink()`](VfsNodeOps::unlink) | Remove the node with the given path | directory |
//! | [`read_dir()`](VfsNodeOps::read_dir) | Read directory entries | directory |
//! | [`is_empty()`](VfsNodeOps::is_empty) | Check if the directory is empty | directory |
//!
//! [inodes]: https://en.wikipedia.org/wiki/Inode

#![no_std]

extern crate alloc;

mod macros;
mod path;
mod structs;

use core::any::Any;

use alloc::sync::Arc;
use axerrno::{ax_err, AxError, AxResult};
use axio::PollState;

pub use self::path::{AbsPath, RelPath};
pub use self::structs::{FileSystemInfo, VfsDirEntry, VfsNodeAttr, VfsNodePerm, VfsNodeType};

/// A wrapper of [`Arc<dyn VfsNodeOps>`].
pub type VfsNodeRef = Arc<dyn VfsNodeOps>;

/// Alias of [`AxError`].
pub type VfsError = AxError;

/// Alias of [`AxResult`].
pub type VfsResult<T = ()> = AxResult<T>;

/// Filesystem operations.
pub trait VfsOps: Send + Sync {
    /// Do something when the filesystem is mounted.
    fn mount(&self, _path: &AbsPath, _mount_point: VfsNodeRef) -> VfsResult {
        Ok(())
    }

    /// Do something when the filesystem is unmounted.
    fn umount(&self) -> VfsResult {
        Ok(())
    }

    /// Format the filesystem.
    fn format(&self) -> VfsResult {
        ax_err!(Unsupported)
    }

    /// Get the attributes of the filesystem.
    fn statfs(&self) -> VfsResult<FileSystemInfo> {
        ax_err!(Unsupported)
    }

    /// Get the root directory of the filesystem.
    fn root_dir(&self) -> VfsNodeRef;
}

/// Node (file/directory/lib) operations.
pub trait VfsNodeOps: Send + Sync {
    /// Do something when the node is opened.
    /// For example, open some special nodes like `/dev/ptmx` should return a new node named `PtyMaster`
    fn open(&self) -> VfsResult<Option<VfsNodeRef>> {
        Ok(None)
    }

    /// Do something when the node is closed.
    fn release(&self) -> VfsResult {
        Ok(())
    }

    /// Get the attributes of the node.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        ax_err!(Unsupported, "get_attr method is unsupported")
    }

    /// Set the mode of the node.
    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        ax_err!(Unsupported, "set_attr method is unsupported")
    }

    /// Get the inode number of the node.
    fn get_inode(&self) -> Option<u64> {
        None
    }

    // file operations:

    /// Read data from the file at the given offset.
    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput, "read_at method InvalidInput")
    }

    /// Write data to the file at the given offset.
    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        ax_err!(InvalidInput, "write_at method InvalidInput")
    }

    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> VfsResult {
        ax_err!(InvalidInput)
    }

    /// Truncate the file to the given size.
    fn truncate(&self, _size: u64) -> VfsResult {
        ax_err!(InvalidInput)
    }

    // directory operations:

    /// Get the parent directory of this directory.
    ///
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<VfsNodeRef> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// Return the node if found.
    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        ax_err!(Unsupported, "lookup method is unsupported in path {}", path)
    }

    /// Create a new node with the given `path` in the directory
    ///
    /// Return [`Ok(())`](Ok) if it already exists.
    fn create(&self, path: &RelPath, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        ax_err!(
            Unsupported,
            "create method is unsupported in path {} type {:?}, mode {:?}",
            path,
            ty,
            mode
        )
    }

    /// Create a new hard link to the src dentry
    fn link(&self, name: &RelPath, _src: Arc<dyn VfsNodeOps>) -> VfsResult<Arc<dyn VfsNodeOps>> {
        ax_err!(Unsupported, "link method is unsupported in path {}", name)
    }

    /// Remove (the hard link of) the node with the given `path` in the directory.
    fn unlink(&self, path: &RelPath) -> VfsResult {
        ax_err!(Unsupported, "unlink method is unsupported in path {}", path)
    }

    /// Rename the node `src_path` to `dst_path` in the directory.
    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult<()> {
        ax_err!(
            Unsupported,
            "rename method is unsupported, src {}, dst {}",
            src_path,
            dst_path
        )
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, start_idx: usize, _dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        ax_err!(
            Unsupported,
            "read_dir method is unsupported, start_idx is {}",
            start_idx
        )
    }

    /// Check if the directory is empty. An empty directory only contains `.` and `..`.
    ///
    /// Brute implementation: read entries and check if there are more than 2.
    fn is_empty(&self) -> VfsResult<bool> {
        let mut buf = [
            VfsDirEntry::default(),
            VfsDirEntry::default(),
            VfsDirEntry::default(),
        ];
        self.read_dir(0, &mut buf).map(|n| n <= 2)
    }

    /// Convert `&self` to [`&dyn Any`][1] that can use
    /// [`Any::downcast_ref`][2].
    ///
    /// [1]: core::any::Any
    /// [2]: core::any::Any#method.downcast_ref
    fn as_any(&self) -> &dyn core::any::Any {
        unimplemented!()
    }

    /// Provides type-erased access to the underlying `Arc` for downcasting.
    fn as_any_arc(self: Arc<Self>) -> Arc<dyn Any + Send + Sync> {
        unimplemented!()
    }

    /// Create a new node with given `path` in the directory, recursively.
    ///
    /// Default implementation `create`s all prefix sub-paths sequentially,
    /// implementor may provide a more efficient impl.
    ///
    /// Return [`Ok(())`](Ok) if already exists.
    fn create_recursive(&self, path: &RelPath, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        for (i, c) in path.char_indices() {
            let part = if c == '/' {
                unsafe { path.get_unchecked(..i) }
            } else {
                continue;
            };
            match self.create(
                &RelPath::new(part),
                VfsNodeType::Dir,
                VfsNodePerm::default_dir(),
            ) {
                Ok(()) | Err(AxError::AlreadyExists) => {}
                err @ Err(_) => return err,
            }
        }
        self.create(path, ty, mode)?;
        Ok(())
    }

    /// Manipulates the underlying device parameters of special files.
    /// In particular, many operating characteristics of character special files
    /// (e.g., terminals) may be controlled with ioctl() requests.
    fn ioctl(&self, _cmd: usize, _arg: usize) -> VfsResult<usize> {
        Err(AxError::Unsupported)
    }

    /// For regular files, the poll() always returns immediately with POLLIN | POLLOUT
    /// events set, since I/O operations on regular files are always considered ready.
    ///
    /// For special files like character devices, poll() requires actual readiness
    /// checks:
    /// - POLLIN is set when the device's input buffer has data available
    /// - POLLOUT is set when the device's output buffer has space available
    /// - POLLHUP is set when peer closed
    fn poll(&self) -> AxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }
}

#[doc(hidden)]
pub mod __priv {
    pub use alloc::sync::Arc;
    pub use axerrno::ax_err;
}
