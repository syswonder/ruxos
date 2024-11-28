/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

extern crate alloc;

use alloc::string::String;
use core::fmt;
use core::mem::MaybeUninit;

use super::FileType;
use crate::io::Result;

use arceos_api::fs as api;

/// Iterator over the entries in a directory.
pub struct ReadDir<'a> {
    path: &'a str,
    inner: api::AxDirHandle,
    buf_pos: usize,
    buf_end: usize,
    end_of_stream: bool,
    dirent_buf: [api::AxDirEntry; 31],
}

/// Entries returned by the [`ReadDir`] iterator.
pub struct DirEntry<'a> {
    dir_path: &'a str,
    entry_name: String,
    entry_type: FileType,
}

/// A builder used to create directories in various manners.
#[derive(Default, Debug)]
pub struct DirBuilder {
    recursive: bool,
}

impl<'a> ReadDir<'a> {
    pub(super) fn new(path: &'a str) -> Result<Self> {
        let mut opts = api::AxOpenOptions::new();
        opts.read(true);
        let inner = api::ax_open_dir(path, &opts)?;

        let dirent_buf: [api::AxDirEntry; 31] = unsafe {
            let mut buf: MaybeUninit<[api::AxDirEntry; 31]> = MaybeUninit::uninit();
            let buf_ptr = buf.as_mut_ptr() as *mut api::AxDirEntry;
            for i in 0..31 {
                buf_ptr.add(i).write(api::AxDirEntry::default());
            }
            buf.assume_init()
        };

        Ok(ReadDir {
            path,
            inner,
            end_of_stream: false,
            buf_pos: 0,
            buf_end: 0,
            dirent_buf,
        })
    }
}

impl<'a> Iterator for ReadDir<'a> {
    type Item = Result<DirEntry<'a>>;

    fn next(&mut self) -> Option<Result<DirEntry<'a>>> {
        if self.end_of_stream {
            return None;
        }

        loop {
            if self.buf_pos >= self.buf_end {
                match api::ax_read_dir(&mut self.inner, &mut self.dirent_buf) {
                    Ok(n) => {
                        if n == 0 {
                            self.end_of_stream = true;
                            return None;
                        }
                        self.buf_pos = 0;
                        self.buf_end = n;
                    }
                    Err(e) => {
                        self.end_of_stream = true;
                        return Some(Err(e));
                    }
                }
            }
            let entry = &self.dirent_buf[self.buf_pos];
            self.buf_pos += 1;
            let entry_name = entry.file_name();
            if entry_name == "." || entry_name == ".." {
                continue;
            }
            let entry_type = entry.file_type();

            return Some(Ok(DirEntry {
                dir_path: self.path,
                entry_name,
                entry_type,
            }));
        }
    }
}

impl<'a> DirEntry<'a> {
    /// Returns the full path to the file that this entry represents.
    ///
    /// The full path is created by joining the original path to `read_dir`
    /// with the filename of this entry.
    pub fn path(&self) -> String {
        String::from(self.dir_path.trim_end_matches('/')) + "/" + &self.entry_name
    }

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

impl fmt::Debug for DirEntry<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("DirEntry").field(&self.path()).finish()
    }
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
    pub fn create<P: AsRef<str>>(&self, path: P) -> Result<()> {
        if self.recursive {
            self.create_dir_all(path)
        } else {
            api::ax_create_dir(path.as_ref())
        }
    }

    /// Creates a new, empty directory at the provided path, recursively creates all parents.
    pub fn create_dir_all<P: AsRef<str>>(&self, path: P) -> Result<()> {
        api::ax_create_dir_all(path.as_ref())
    }
}
