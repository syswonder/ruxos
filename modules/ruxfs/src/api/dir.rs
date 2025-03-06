/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{borrow::ToOwned, string::String, vec};
use axerrno::ax_err;
use axfs_vfs::{AbsPath, VfsDirEntry, VfsError};
use axio::Result;
use core::{fmt, iter::Iterator, str};

use super::{FileAttr, FileType};
use crate::fops;

/// A wrapped directory type.
///
/// Provides a way to open a directory and iterate over its contents.
pub struct Directory {
    inner: fops::Directory,
}

impl Directory {
    /// Opens a directory for reading entries.
    pub fn open(path: &AbsPath) -> Result<Self> {
        let node = fops::lookup(path)?;
        let inner = fops::open_dir(path, node, &fops::OpenOptions::new())?;
        Ok(Self { inner })
    }

    /// Get attributes of the directory.
    pub fn get_attr(&self) -> Result<FileAttr> {
        self.inner.get_attr()
    }

    /// Reads directory entries starts from the current position into the
    /// given buffer, returns the number of entries read.
    ///
    /// After the read, the cursor of the directory will be advanced by the
    /// number of entries read.
    pub fn read_dir(&mut self, buf: &mut [DirEntry]) -> Result<usize> {
        let mut buffer = vec![fops::DirEntry::default(); buf.len()];
        let len = self.inner.read_dir(&mut buffer)?;
        for (i, entry) in buffer.iter().enumerate().take(len) {
            buf[i] = DirEntry {
                entry_name: unsafe { str::from_utf8_unchecked(entry.name_as_bytes()).to_owned() },
                entry_type: entry.entry_type(),
            };
        }
        Ok(len)
    }
}

/// Implements the iterator trait for the directory.
impl Iterator for Directory {
    type Item = Result<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = [VfsDirEntry::default()];
        match self.inner.read_dir(buf.as_mut_slice()) {
            Ok(0) => None,
            Ok(1) => Some(Ok(DirEntry {
                entry_name: unsafe { str::from_utf8_unchecked(buf[0].name_as_bytes()).to_owned() },
                entry_type: buf[0].entry_type(),
            })),
            Ok(_) => unreachable!(),
            Err(e) => Some(Err(e)),
        }
    }
}

/// Entry type used by `Directory::read_dir`.
#[derive(Default, Clone)]
pub struct DirEntry {
    entry_name: String,
    entry_type: FileType,
}

impl DirEntry {
    /// Returns the bare file name of this directory entry without any other
    /// leading path component.
    pub fn file_name(&self) -> String {
        self.entry_name.clone()
    }

    /// Returns the file type for the file that this entry points at.
    pub fn file_type(&self) -> FileType {
        self.entry_type
    }
}

impl fmt::Debug for DirEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DirEntry").field(&self.file_name()).finish()
    }
}

/// A builder used to create directories in various manners.
#[derive(Default, Debug)]
pub struct DirBuilder {
    recursive: bool,
}

impl DirBuilder {
    /// Creates a new set of options with default mode/security settings for all
    /// platforms and also non-recursive.
    pub fn new() -> Self {
        Self { recursive: false }
    }

    /// Indicates that directories should be created recursively, creating all
    /// parent directories. Parents that do not exist are created with the same
    /// security and permissions settings.
    pub fn recursive(&mut self, recursive: bool) -> &mut Self {
        self.recursive = recursive;
        self
    }

    /// Creates the specified directory with the options configured in this
    /// builder.
    pub fn create(&self, path: &AbsPath) -> Result<()> {
        if self.recursive {
            self.create_dir_all(path)
        } else {
            match fops::lookup(path) {
                Ok(_) => return ax_err!(AlreadyExists),
                Err(VfsError::NotFound) => {}
                Err(e) => return ax_err!(e),
            }
            fops::create_dir(path)
        }
    }

    /// Recursively create a directory and all of its parent components if they
    /// are missing.
    pub fn create_dir_all(&self, path: &AbsPath) -> Result<()> {
        match fops::lookup(path) {
            Ok(_) => return ax_err!(AlreadyExists),
            Err(VfsError::NotFound) => {}
            Err(e) => return ax_err!(e),
        }
        fops::create_dir_all(path)
    }
}
