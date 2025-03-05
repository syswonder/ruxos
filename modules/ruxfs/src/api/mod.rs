/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! High-level filesystem manipulation operations.
//!
//! Provided for `arceos_api` module and `axstd` user lib.

mod dir;
mod file;

use alloc::{string::String, vec::Vec};
use axerrno::ax_err;
use axfs_vfs::{AbsPath, VfsError};
use axio::{self as io, prelude::*};

use crate::fops;

// Export high-level directory-related types.
pub use dir::{DirBuilder, DirEntry, Directory};

// Export high-level file-related types.
pub use file::{File, FileAttr, FilePerm, FileType, OpenOptions};

/// Returns the current working directory as a [`AbsPath`].
pub fn current_dir() -> io::Result<AbsPath<'static>> {
    Ok(fops::current_dir().unwrap())
}

/// Changes the current working directory to the specified path.
pub fn set_current_dir(path: AbsPath<'static>) -> io::Result<()> {
    fops::set_current_dir(path)
}

/// Return the canonicalized and absolute path of the specified path.
pub fn absolute_path(path: &str) -> io::Result<AbsPath<'static>> {
    fops::absolute_path(path)
}

/// Get the attibutes of a file or directory.
pub fn get_attr(path: &AbsPath) -> io::Result<FileAttr> {
    fops::lookup(path)?.get_attr()
}

/// Read the entire contents of a file into a bytes vector.
pub fn read(path: &AbsPath) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let size = file.get_attr().map(|m| m.size()).unwrap_or(0);
    let mut bytes = Vec::with_capacity(size as usize);
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
pub fn read_to_string(path: &AbsPath) -> io::Result<String> {
    let mut file = File::open(path)?;
    let size = file.get_attr().map(|m| m.size()).unwrap_or(0);
    let mut string = String::with_capacity(size as usize);
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
pub fn write<C: AsRef<[u8]>>(path: &AbsPath, contents: C) -> io::Result<()> {
    File::create(path)?.write_all(contents.as_ref())
}

/// Creates a new, empty directory at the provided path.
pub fn create_dir(path: &AbsPath) -> io::Result<()> {
    DirBuilder::new().create(path)
}

/// Recursively create a directory and all of its parent components if they
/// are missing.
pub fn create_dir_all(path: &AbsPath) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(path)
}

/// Removes an empty directory.
pub fn remove_dir(path: &AbsPath) -> io::Result<()> {
    let node = fops::lookup(path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        return ax_err!(NotADirectory);
    }
    if fops::is_mount_point(path) {
        return ax_err!(PermissionDenied);
    }
    if !attr.perm().owner_writable() {
        return ax_err!(PermissionDenied);
    }
    if !node.is_empty()? {
        return ax_err!(DirectoryNotEmpty);
    }
    fops::remove_dir(path)
}

/// Removes a file from the filesystem.
pub fn remove_file(path: &AbsPath) -> io::Result<()> {
    let node = fops::lookup(path)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        return ax_err!(IsADirectory);
    }
    if !attr.perm().owner_writable() {
        return ax_err!(PermissionDenied);
    }
    fops::remove_file(path)
}

/// Rename a file or directory to a new name.
/// Delete the original file if `old` already exists.
///
/// This only works then the new path is in the same mounted fs.
pub fn rename(old: &AbsPath, new: &AbsPath) -> io::Result<()> {
    fops::lookup(old)?;
    match fops::lookup(new) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(VfsError::NotFound) => fops::rename(old, new),
        Err(e) => ax_err!(e),
    }
}
