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
use ruxfs::{
    api::{Directory, File},
    AbsPath, RelPath,
};
use axio::{Read, Write, Seek};

pub use axio::SeekFrom as AxSeekFrom;
pub use ruxfs::api::DirEntry as AxDirEntry;
pub use ruxfs::api::FileAttr as AxFileAttr;
pub use ruxfs::api::FilePerm as AxFilePerm;
pub use ruxfs::api::FileType as AxFileType;
pub use ruxfs::api::OpenOptions as AxOpenOptions;

#[cfg(feature = "blkfs")]
pub use ruxfs::dev::Disk as AxDisk;

#[cfg(feature = "myfs")]
pub use ruxfs::MyFileSystemIf;

/// A handle to an opened file.
pub struct AxFileHandle(File);

/// A handle to an opened directory.
pub struct AxDirHandle(Directory);

pub fn ax_open_file(path: &str, opts: &AxOpenOptions) -> AxResult<AxFileHandle> {
    Ok(AxFileHandle(opts.open(&parse_path(path)?)?))
}

pub fn ax_open_dir(path: &str, _opts: &AxOpenOptions) -> AxResult<AxDirHandle> {
    Ok(AxDirHandle(Directory::open(&parse_path(path)?)?))
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