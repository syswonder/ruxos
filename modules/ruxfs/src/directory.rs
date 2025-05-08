/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{AbsPath, DirEntry, FileAttr};
use alloc::sync::Arc;
use axerrno::{AxResult, LinuxError, LinuxResult};
use axfs_vfs::VfsNodeRef;
use axio::PollState;
use capability::{Cap, WithCap};
use ruxfdtable::{FileLike, OpenFlags, RuxStat};
use spin::rwlock::RwLock;
/// An opened directory object, with open permissions and a cursor for entry reading.
///
/// Providing entry reading operations.
pub struct Directory {
    path: AbsPath<'static>,
    node: WithCap<VfsNodeRef>,
    entry_idx: AtomicUsize,
    flags: RwLock<OpenFlags>,
}

impl Directory {
    /// Creates an opened directory.
    pub fn new(path: AbsPath<'static>, node: VfsNodeRef, flags: OpenFlags) -> Self {
        Self {
            path,
            node: WithCap::new(node, Cap::from(flags) | Cap::EXECUTE),
            entry_idx: AtomicUsize::new(0),
            flags: RwLock::new(flags),
        }
    }
    /// Get the entry cursor of the directory.
    pub fn entry_idx(&self) -> usize {
        self.entry_idx.load(Ordering::Relaxed)
    }

    /// Set the entry cursor of the directory.
    pub fn set_entry_idx(&self, idx: usize) {
        self.entry_idx.store(idx, Ordering::Relaxed);
    }
    /// Gets the absolute path of the directory.
    pub fn path(&self) -> AbsPath {
        self.path.clone()
    }

    /// Gets the file attributes.
    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.access(Cap::empty())?.get_attr()
    }

    /// Reads directory entries starts from the current position into the
    /// given buffer. Returns the number of entries read.
    ///
    /// After the read, the cursor will be advanced by the number of entries
    /// read.
    pub fn read_dir(&self, dirents: &mut [DirEntry]) -> AxResult<usize> {
        let current_entry_idx = self.entry_idx();
        let n = self
            .node
            .access(Cap::EXECUTE)?
            .read_dir(current_entry_idx, dirents)?;
        self.set_entry_idx(current_entry_idx + n);
        Ok(n)
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        unsafe { self.node.access_unchecked().release().ok() };
    }
}

impl FileLike for Directory {
    fn path(&self) -> AbsPath {
        self.path.clone()
    }

    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EISDIR)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EISDIR)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        Ok(RuxStat::from(self.get_attr()?))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }

    fn flags(&self) -> OpenFlags {
        *self.flags.read()
    }

    fn set_flags(&self, flags: OpenFlags) -> LinuxResult {
        *self.flags.write() = flags;
        Ok(())
    }
}

/// Implements the iterator trait for the directory.
impl Iterator for Directory {
    type Item = AxResult<DirEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        use core::str;
        let mut buf = [DirEntry::default()];
        match self.read_dir(buf.as_mut_slice()) {
            Ok(0) => None,
            Ok(1) => Some(Ok(DirEntry::new(
                unsafe { str::from_utf8_unchecked(buf[0].name_as_bytes()) },
                buf[0].entry_type(),
            ))),
            Ok(_) => unreachable!(),
            Err(e) => Some(Err(e)),
        }
    }
}
