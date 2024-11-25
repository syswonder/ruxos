/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::fops::{self, current_dir};
use axerrno::ax_err;
use axfs_vfs::{AbsPath, RelPath, VfsError};
use axio::{prelude::*, Result, SeekFrom};
use capability::Cap;
use core::fmt;

/// A structure representing a type of file with accessors for each file type.
/// It is returned by [`Metadata::file_type`] method.
pub type FileType = fops::FileType;

/// Representation of the various permissions on a file.
pub type Permissions = fops::FilePerm;

/// An object providing access to an open file on the filesystem.
pub struct File {
    inner: fops::File,
}

/// Metadata information about a file.
pub struct Metadata(fops::FileAttr);

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone)]
pub struct OpenOptions {
    // generic
    read: bool,
    write: bool,
    append: bool,
    truncate: bool,
    create: bool,
    create_new: bool,
    // system-specific
    _custom_flags: i32,
    _mode: u32,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub const fn new() -> Self {
        Self {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            _custom_flags: 0,
            _mode: 0o666,
        }
    }

    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) -> &mut Self {
        self.read = read;
        self
    }

    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) -> &mut Self {
        self.write = write;
        self
    }

    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) -> &mut Self {
        self.append = append;
        self
    }

    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) -> &mut Self {
        self.truncate = truncate;
        self
    }

    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) -> &mut Self {
        self.create = create;
        self
    }

    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) -> &mut Self {
        self.create_new = create_new;
        self
    }

    /// Check if the options are valid.
    pub const fn is_valid(&self) -> bool {
        if !self.read && !self.write && !self.append {
            return false;
        }
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return false;
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return false;
                }
            }
        }
        true
    }

    /// Opens a file at `path` with the options specified by `self`.
    pub fn open(&self, path: &str) -> Result<File> {
        // Check options
        if !self.is_valid() {
            return ax_err!(InvalidInput);
        }
        // Find node
        let path = if path.starts_with("/") {
            AbsPath::new_canonicalized(path)
        } else {
            current_dir().join(&RelPath::new_canonicalized(path))
        };
        // Check flag and attr
        let node = match fops::lookup(&path) {
            Ok(node) => {
                if self.create_new {
                    return ax_err!(AlreadyExists);
                }
                node
            }
            Err(VfsError::NotFound) => {
                if !self.create && !self.create_new {
                    return ax_err!(NotFound);
                }
                fops::create_file(&path)?;
                fops::lookup(&path)?
            }
            Err(e) => return Err(e),
        };
        if node.get_attr()?.is_dir() {
            return ax_err!(IsADirectory);
        }
        // Truncate
        if self.truncate {
            node.truncate(0)?;
        }
        // Open
        fops::open_file(&path, node, self.into(), self.append).map(|inner| File { inner })
    }
}

impl From<&OpenOptions> for Cap {
    fn from(opts: &OpenOptions) -> Cap {
        let mut cap = Cap::empty();
        if opts.read {
            cap |= Cap::READ;
        }
        if opts.write | opts.append {
            cap |= Cap::WRITE;
        }
        cap
    }
}

impl fmt::Debug for OpenOptions {
    #[allow(unused_assignments)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut written = false;
        macro_rules! fmt_opt {
            ($field: ident, $label: literal) => {
                if self.$field {
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

impl Metadata {
    /// Returns the file type for this metadata.
    pub const fn file_type(&self) -> FileType {
        self.0.file_type()
    }

    /// Returns `true` if this metadata is for a directory. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_file`].
    pub const fn is_dir(&self) -> bool {
        self.0.is_dir()
    }

    /// Returns `true` if this metadata is for a regular file. The
    /// result is mutually exclusive to the result of
    /// [`Metadata::is_dir`].
    pub const fn is_file(&self) -> bool {
        self.0.is_file()
    }

    /// Returns the size of the file, in bytes, this metadata is for.
    #[allow(clippy::len_without_is_empty)]
    pub fn len(&self) -> u64 {
        self.0.size()
    }

    /// Returns the permissions of the file this metadata is for.
    pub const fn permissions(&self) -> Permissions {
        self.0.perm()
    }

    /// Returns the total size of this file in bytes.
    pub const fn size(&self) -> u64 {
        self.0.size()
    }

    /// Returns the number of blocks allocated to the file, in 512-byte units.
    pub const fn blocks(&self) -> u64 {
        self.0.blocks()
    }
}

impl fmt::Debug for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metadata")
            .field("file_type", &self.file_type())
            .field("is_dir", &self.is_dir())
            .field("is_file", &self.is_file())
            .field("permissions", &self.permissions())
            .finish_non_exhaustive()
    }
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

    /// Queries metadata about the underlying file.
    pub fn metadata(&self) -> Result<Metadata> {
        self.inner.get_attr().map(Metadata)
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
