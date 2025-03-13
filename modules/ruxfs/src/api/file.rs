/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axerrno::ax_err;
use axfs_vfs::{AbsPath, VfsError};
use axio::{prelude::*, Result, SeekFrom};
use core::fmt;

use crate::fops;

/// A structure representing a type of file with accessors for each file type
pub type FileType = fops::FileType;

/// Representation of the various permissions on a file.
pub type FilePerm = fops::FilePerm;

/// A structure representing the attributes of a file.
pub type FileAttr = fops::FileAttr;

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone)]
pub struct OpenOptions(fops::OpenOptions);

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub const fn new() -> Self {
        Self(fops::OpenOptions::new())
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.0.read(read);
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.0.write(write);
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.0.append(append);
        self
    }

    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.0.truncate(truncate);
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.0.create(create);
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.0.create_new(create_new);
        self
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub fn open(&self, path: &AbsPath) -> Result<File> {
        // Check options
        if !self.0.is_valid() {
            return ax_err!(InvalidInput);
        }
        // Find node, check flag and attr
        let node = match fops::lookup(path) {
            Ok(node) => {
                if self.0.create_new {
                    return ax_err!(AlreadyExists);
                }
                node
            }
            Err(VfsError::NotFound) => {
                if !self.0.create && !self.0.create_new {
                    return ax_err!(NotFound);
                }
                fops::create_file(path)?;
                fops::lookup(path)?
            }
            Err(e) => return Err(e),
        };
        if node.get_attr()?.is_dir() {
            return ax_err!(IsADirectory);
        }
        // Truncate
        if self.0.truncate {
            node.truncate(0)?;
        }
        // Open
        fops::open_file(path, node, &self.0).map(|inner| File { inner })
    }
}

impl fmt::Debug for OpenOptions {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;
        macro_rules! fmt_opt {
            ($field: ident, $label: literal) => {
                if self.0.$field {
                    if written {
                        write!(f, " | ")?;
                    }
                    write!(f, $label)?;
                    written = true;
                }
            };
        }
        fmt_opt!(read, "READ");
        fmt_opt!(write, "WRITE");
        fmt_opt!(append, "APPEND");
        fmt_opt!(truncate, "TRUNC");
        fmt_opt!(create, "CREATE");
        fmt_opt!(create_new, "CREATE_NEW");
        Ok(())
    }
}

/// An object providing access to an open file on the filesystem.
pub struct File {
    inner: fops::File,
}

impl File {
    /// Attempts to open a file in read-only mode.
    pub fn open(path: &AbsPath) -> Result<Self> {
        OpenOptions::new().read(true).open(path)
    }

    /// Opens a file in write-only mode.
    pub fn create(path: &AbsPath) -> Result<Self> {
        OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
    }

    /// Creates a new file in read-write mode; error if the file exists.
    pub fn create_new(path: &AbsPath) -> Result<Self> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(path)
    }

    /// Returns a new OpenOptions object.
    pub fn options() -> OpenOptions {
        OpenOptions::new()
    }

    /// Truncates or extends the underlying file, updating the size of
    /// this file to become `size`.
    pub fn set_len(&self, size: u64) -> Result<()> {
        self.inner.truncate(size)
    }

    /// Truncates the file to 0 length.
    pub fn truncate(&self, size: u64) -> Result<()> {
        self.inner.truncate(size)
    }

    /// Flush the buffered contents to disk.
    pub fn flush(&self) -> Result<()> {
        self.inner.flush()
    }

    /// Get the attributes of the file.
    pub fn get_attr(&self) -> Result<FileAttr> {
        self.inner.get_attr()
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.inner.read(buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> Result<()> {
        self.inner.flush()
    }
}

impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64> {
        self.inner.seek(pos)
    }
}
