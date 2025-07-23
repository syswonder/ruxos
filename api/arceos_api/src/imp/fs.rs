/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::string::String;
use axerrno::AxResult;
pub use axio::SeekFrom as AxSeekFrom;
use axio::{Read, Seek, Write};
pub use ruxfs::api::DirEntry as AxDirEntry;
pub use ruxfs::api::FileAttr as AxFileAttr;
pub use ruxfs::api::FilePerm as AxFilePerm;
pub use ruxfs::api::FileType as AxFileType;
use ruxfs::api::{open_dir, open_file, AbsPath, Directory, File, OpenFlags, RelPath};
#[derive(Clone, Debug)]
/// arceos api OpenOptions
pub struct AxOpenOptions {
    /// Open for reading.
    pub read: bool,
    /// Open for writing.
    pub write: bool,
    /// Append to the end of the file.
    pub append: bool,
    /// Truncate the file to zero length.
    pub truncate: bool,
    /// Create a new file.
    pub create: bool,
    /// Create a new file, failing if it already exists.
    pub create_new: bool,
    // system-specific
    _custom_flags: i32,
    _mode: u32,
}

impl Default for AxOpenOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl AxOpenOptions {
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
    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    /// Get flags access mode
    fn get_access_mode(&self) -> AxResult<OpenFlags> {
        match (self.read, self.write) {
            (true, false) => Ok(OpenFlags::empty()),
            (false, true) => Ok(OpenFlags::O_WRONLY),
            (true, true) => Ok(OpenFlags::O_RDWR),
            (false, false) => Err(axio::Error::InvalidInput),
        }
    }

    /// Get flags creation mode
    fn get_creation_mode(&self) -> AxResult<OpenFlags> {
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return Err(axio::Error::InvalidInput);
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return Err(axio::Error::InvalidInput);
                }
            }
        }

        Ok(match (self.create, self.truncate, self.create_new) {
            (false, false, false) => OpenFlags::empty(),
            (true, false, false) => OpenFlags::O_CREAT,
            (false, true, false) => OpenFlags::O_TRUNC,
            (true, true, false) => OpenFlags::O_CREAT | OpenFlags::O_TRUNC,
            (_, _, true) => OpenFlags::O_CREAT | OpenFlags::O_EXCL,
        })
    }
}

impl TryFrom<AxOpenOptions> for OpenFlags {
    type Error = axerrno::AxError;

    fn try_from(opt: AxOpenOptions) -> Result<Self, Self::Error> {
        Ok(opt.get_access_mode()? | opt.get_creation_mode()?)
    }
}

#[cfg(feature = "blkfs")]
pub use ruxfs::dev::Disk as AxDisk;

#[cfg(feature = "myfs")]
pub use ruxfs::MyFileSystemIf;
use ruxtask::fs::{FileSystem, InitFs};

/// A handle to an opened file.
pub struct AxFileHandle(File);

/// A handle to an opened directory.
pub struct AxDirHandle(Directory);

pub fn ax_open_file(path: &str, opts: &AxOpenOptions) -> AxResult<AxFileHandle> {
    let file = open_file(&parse_path(path)?, OpenFlags::try_from(opts.clone())?)?;
    Ok(AxFileHandle(file))
}

pub fn ax_open_dir(path: &str, opts: &AxOpenOptions) -> AxResult<AxDirHandle> {
    let dir = open_dir(&parse_path(path)?, OpenFlags::try_from(opts.clone())?)?;
    Ok(AxDirHandle(dir))
}

pub fn ax_get_attr(path: &str) -> AxResult<AxFileAttr> {
    ruxfs::api::get_attr(&parse_path(path)?)
}

pub fn ax_read_file(file: &mut AxFileHandle, buf: &mut [u8]) -> AxResult<usize> {
    file.0.read(buf)
}

pub fn ax_write_file(file: &mut AxFileHandle, buf: &[u8]) -> AxResult<usize> {
    file.0.write(buf)
}

pub fn ax_truncate_file(file: &AxFileHandle, size: u64) -> AxResult {
    file.0.truncate(size)
}

pub fn ax_flush_file(file: &AxFileHandle) -> AxResult {
    file.0.flush()
}

pub fn ax_seek_file(file: &mut AxFileHandle, pos: AxSeekFrom) -> AxResult<u64> {
    file.0.seek(pos)
}

pub fn ax_file_attr(file: &AxFileHandle) -> AxResult<AxFileAttr> {
    file.0.get_attr()
}

pub fn ax_read_dir(dir: &mut AxDirHandle, dirents: &mut [AxDirEntry]) -> AxResult<usize> {
    dir.0.read_dir(dirents)
}

pub fn ax_create_dir(path: &str) -> AxResult {
    ruxfs::api::create_dir(&parse_path(path)?)
}

pub fn ax_create_dir_all(path: &str) -> AxResult {
    ruxfs::api::create_dir_all(&parse_path(path)?)
}

pub fn ax_remove_dir(path: &str) -> AxResult {
    ruxfs::api::remove_dir(&parse_path(path)?)
}

pub fn ax_remove_file(path: &str) -> AxResult {
    ruxfs::api::remove_file(&parse_path(path)?)
}

pub fn ax_rename(old: &str, new: &str) -> AxResult {
    ruxfs::api::rename(&parse_path(old)?, &parse_path(new)?)
}

pub fn ax_current_dir() -> AxResult<String> {
    ruxfs::api::current_dir().map(|path| path.to_string())
}

pub fn ax_set_current_dir(path: &str) -> AxResult {
    ruxfs::api::set_current_dir(parse_path(path)?)
}

fn parse_path(path: &str) -> AxResult<AbsPath<'static>> {
    if path.starts_with('/') {
        Ok(AbsPath::new_canonicalized(path))
    } else {
        ruxfs::api::current_dir().map(|cwd| cwd.join(&RelPath::new_canonicalized(path)))
    }
}

impl Iterator for AxDirHandle {
    type Item = AxResult<AxDirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
