/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Filesystem manipulation operations.

mod dir;
mod file;

use crate::io::{self, prelude::*};

#[cfg(feature = "alloc")]
use alloc::{string::String, vec::Vec};

pub use self::dir::{DirBuilder, DirEntry, ReadDir};
pub use self::file::{File, FileType, Metadata, OpenOptions, Permissions};

/// Read the entire contents of a file into a bytes vector.
#[cfg(feature = "alloc")]
pub fn read<P: AsRef<str>>(path: P) -> io::Result<Vec<u8>> {
    let mut file = File::open(path.as_ref())?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut bytes = Vec::with_capacity(size as usize);
    file.read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Read the entire contents of a file into a string.
#[cfg(feature = "alloc")]
pub fn read_to_string<P: AsRef<str>>(path: P) -> io::Result<String> {
    let mut file = File::open(path.as_ref())?;
    let size = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mut string = String::with_capacity(size as usize);
    file.read_to_string(&mut string)?;
    Ok(string)
}

/// Write a slice as the entire contents of a file.
pub fn write<P: AsRef<str>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
    File::create(path.as_ref())?.write_all(contents.as_ref())
}

/// Given a path, query the file system to get information about a file,
/// directory, etc.
pub fn metadata<P: AsRef<str>>(path: P) -> io::Result<Metadata> {
    File::open(path.as_ref())?.metadata()
}

/// Returns an iterator over the entries within a directory.
pub fn read_dir(path: &str) -> io::Result<ReadDir<'_>> {
    ReadDir::new(path)
}

/// Creates a new, empty directory at the provided path.
pub fn create_dir<P: AsRef<str>>(path: P) -> io::Result<()> {
    DirBuilder::new().create(path.as_ref())
}

/// Recursively create a directory and all of its parent components if they
/// are missing.
pub fn create_dir_all<P: AsRef<str>>(path: P) -> io::Result<()> {
    DirBuilder::new().recursive(true).create(path.as_ref())
}

/// Removes an empty directory.
pub fn remove_dir<P: AsRef<str>>(path: P) -> io::Result<()> {
    arceos_api::fs::ax_remove_dir(path.as_ref())
}

/// Removes a file from the filesystem.
pub fn remove_file<P: AsRef<str>>(path: P) -> io::Result<()> {
    arceos_api::fs::ax_remove_file(path.as_ref())
}

/// Rename a file or directory to a new name.
/// Delete the original file if `old` already exists.
///
/// This only works then the new path is in the same mounted fs.
pub fn rename<P: AsRef<str>>(old: P, new: P) -> io::Result<()> {
    arceos_api::fs::ax_rename(old.as_ref(), new.as_ref())
}
