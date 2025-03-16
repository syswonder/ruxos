/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Low-level filesystem operations. Provided for `ruxfs::api` and `ruxos_posix_api::fs` modules.
//!
//! - File: open, read, write, seek, truncate
//! - Directory: open, read, create, remove
//!
//! The interface is designed with low coupling to avoid repetitive error handling.
use alloc::{sync::Arc, vec::Vec};
use axerrno::{AxError, AxResult};
use axfs_vfs::{AbsPath, RelPath, VfsNodeOps, VfsNodeRef, VfsNodeType};
use capability::Cap;
use ruxfdtable::{FileLike, OpenFlags};

use crate::{
    directory::Directory,
    file::File,
    root::{MountPoint, RootDirectory},
    FileAttr,
};

#[crate_interface::def_interface]
/// Current working directory operations.
pub trait CurrentWorkingDirectoryOps {
    /// Initializes the root filesystem with the specified mount points.
    fn init_rootfs(mount_points: Vec<MountPoint>);
    /// Returns the parent node of the specified path.
    fn parent_node_of(dir: Option<&VfsNodeRef>, path: &RelPath) -> VfsNodeRef;
    /// Returns the absolute path of the specified path.
    fn absolute_path(path: &str) -> AxResult<AbsPath<'static>>;
    /// Returns the current working directory.
    fn current_dir() -> AxResult<AbsPath<'static>>;
    /// Sets the current working directory.
    fn set_current_dir(path: AbsPath<'static>) -> AxResult;
    /// get the root directory of the filesystem
    fn root_dir() -> Arc<RootDirectory>;
}

#[allow(unused)]
pub(crate) fn absolute_path(path: &str) -> AxResult<AbsPath<'static>> {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::absolute_path, path)
}

/// Get the current working directory.
pub fn current_dir() -> AxResult<AbsPath<'static>> {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::current_dir)
}

/// Set the current working directory.
pub fn set_current_dir(path: AbsPath<'static>) -> AxResult {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::set_current_dir, path)
}

pub(crate) fn init_rootfs(mount_points: Vec<MountPoint>) {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::init_rootfs, mount_points)
}

pub(crate) fn root_dir() -> Arc<RootDirectory> {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::root_dir)
}

/* File operations with absolute path. */

/// Look up a file given an absolute path.
pub fn lookup(path: &AbsPath) -> AxResult<VfsNodeRef> {
    root_dir().clone().lookup(&path.to_rel())
}

/// Get the file attributes given an absolute path.
pub fn get_attr(path: &AbsPath) -> AxResult<FileAttr> {
    lookup(path)?.get_attr()
}

/// PERF: resolve the same path three times if not found!
///
/// Internal helper to open a file by absolute path with flags and validation.
pub(crate) fn open_abspath(path: &AbsPath, flags: OpenFlags) -> AxResult<VfsNodeRef> {
    let node = match lookup(path) {
        Ok(node) => {
            if flags.contains(OpenFlags::O_EXCL | OpenFlags::O_CREAT) {
                return Err(AxError::AlreadyExists);
            }
            let attr = node.get_attr()?;
            if !attr.is_dir() && flags.contains(OpenFlags::O_DIRECTORY) {
                return Err(AxError::NotADirectory);
            }
            if attr.is_file() && flags.contains(OpenFlags::O_TRUNC) {
                node.truncate(0)?;
            }
            node
        }
        Err(AxError::NotFound) => {
            if !flags.contains(OpenFlags::O_CREAT) || flags.contains(OpenFlags::O_DIRECTORY) {
                return Err(AxError::NotFound);
            }
            create_file(path)?;
            lookup(path)?
        }
        Err(e) => return Err(e),
    };
    let attr = node.get_attr()?;
    if !Cap::from(attr.perm()).contains(Cap::from(flags)) {
        return Err(AxError::PermissionDenied);
    }
    if let Some(new_node) = node.open()? {
        return Ok(new_node);
    }
    Ok(node)
}

/// Opens a file-like object (file or directory) at given path with flags.
pub fn open_file_like(path: &AbsPath, flags: OpenFlags) -> AxResult<Arc<dyn FileLike>> {
    let node = open_abspath(path, flags)?;
    if node.get_attr()?.is_dir() {
        Ok(Arc::new(Directory::new(path.to_owned(), node, flags)))
    } else {
        Ok(Arc::new(File::new(path.to_owned(), node, flags)))
    }
}

/// Create a file given an absolute path.
///
/// This function will not check if the file exists, check it with [`lookup`] first.
pub fn create_file(path: &AbsPath) -> AxResult {
    root_dir().create(&path.to_rel(), VfsNodeType::File)
}

/// Create a directory given an absolute path.
///
/// This function will not check if the directory exists, check it with [`lookup`] first.
pub fn create_dir(path: &AbsPath) -> AxResult {
    root_dir().create(&path.to_rel(), VfsNodeType::Dir)
}

/// Create a directory recursively given an absolute path.
///
/// This function will not check if the directory exists, check it with [`lookup`] first.
pub fn create_dir_all(path: &AbsPath) -> AxResult {
    root_dir().create_recursive(&path.to_rel(), VfsNodeType::Dir)
}

/// Remove a file given an absolute path.
///
/// This function will not check if the file exits or removeable,
/// check it with [`lookup`] first.
pub fn remove_file(path: &AbsPath) -> AxResult {
    root_dir().unlink(&path.to_rel())
}

/// Remove a directory given an absolute path.
///
/// This function will not check if the directory exists or is empty,
/// check it with [`lookup`] first.
pub fn remove_dir(path: &AbsPath) -> AxResult {
    root_dir().unlink(&path.to_rel())
}

/// Check if a directory is a mount point.
pub fn is_mount_point(path: &AbsPath) -> bool {
    root_dir().contains(path)
}

/// Rename a file given an old and a new absolute path.
///
/// This function will not check if the old path or new path exists, check it with
/// [`lookup`] first.
pub fn rename(old: &AbsPath, new: &AbsPath) -> AxResult {
    root_dir().rename(&old.to_rel(), &new.to_rel())
}
