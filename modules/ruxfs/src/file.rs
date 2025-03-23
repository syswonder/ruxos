/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use alloc::sync::Arc;
use axerrno::{ax_err_type, AxResult, LinuxError, LinuxResult};
use axfs_vfs::VfsNodeRef;
use axio::{PollState, Read, SeekFrom, Write};
use capability::{Cap, WithCap};

use ruxfdtable::{FileLike, OpenFlags, RuxStat};
use spin::{mutex::Mutex, RwLock};

use crate::{AbsPath, FileAttr, FileType};

/// An opened file with permissions and a cursor for I/O operations.
pub struct File {
    /// Absolute path to the file
    path: AbsPath<'static>,
    /// Underlying VFS node with capabilities
    node: WithCap<VfsNodeRef>,
    /// Current read/write offset.
    ///
    /// Note: Operations like read/write/seek on `node` require atomic updates to `offset`.  
    /// Using `AtomicU64` alone cannot lock both `node` and `offset` atomically, risking inconsistent state.  
    /// `Mutex` ensures `offset` and `node` are modified as a single atomic unit.  
    offset: Mutex<u64>,
    /// File mode flags
    flags: RwLock<OpenFlags>,
}

impl File {
    /// Create an opened file.
    pub fn new(path: AbsPath<'static>, node: VfsNodeRef, flags: OpenFlags) -> Self {
        Self {
            path,
            node: WithCap::new(node, Cap::from(flags)),
            offset: Mutex::new(0),
            flags: RwLock::new(flags),
        }
    }

    /// Reads data into `dst` from current offset. Atomically updates the offset  
    /// after reading. Locking ensures synchronization with underlying node operations.
    fn read(&self, dst: &mut [u8]) -> AxResult<usize> {
        let mut offset = self.offset.lock();
        let read_len = self.read_at(*offset, dst)?;
        *offset += read_len as u64;
        Ok(read_len)
    }

    /// Writes data from `src` to current offset. Handles append mode (O_APPEND) by  
    /// resetting offset to file size. Atomically updates offset after writing.
    fn write(&self, src: &[u8]) -> AxResult<usize> {
        let mut offset = self.offset.lock();
        if self.flags.read().contains(OpenFlags::O_APPEND) {
            *offset = self.get_attr()?.size();
        };
        let node = self.node.access(Cap::WRITE)?;
        let write_len = node.write_at(*offset, src)?;
        *offset += write_len as u64;
        Ok(write_len)
    }

    /// Get the abcolute path of the file.
    pub fn path(&self) -> AbsPath {
        self.path.clone()
    }

    /// Gets the file attributes.
    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.access(Cap::empty())?.get_attr()
    }

    /// Truncates the file to the specified size.
    pub fn truncate(&self, size: u64) -> AxResult {
        self.node.access(Cap::WRITE)?.truncate(size)
    }

    /// Reads the file at the given position. Returns the number of bytes read.
    ///
    /// It does not update the file cursor.
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> AxResult<usize> {
        self.node.access(Cap::READ)?.read_at(offset, buf)
    }

    /// Writes the file at the given position. Returns the number of bytes
    /// written.
    ///
    /// It does not update the file cursor.
    pub fn write_at(&self, offset: u64, buf: &[u8]) -> AxResult<usize> {
        self.node.access(Cap::WRITE)?.write_at(offset, buf)
    }

    /// Flushes the file, writes all buffered data to the underlying device.
    pub fn flush(&self) -> AxResult {
        self.node.access(Cap::WRITE)?.fsync()
    }

    /// Sets the cursor of the file to the specified offset. Returns the new
    /// position after the seek.
    pub fn seek(&self, pos: SeekFrom) -> AxResult<u64> {
        let size = self.get_attr()?.size();
        let mut offset = self.offset.lock();
        let new_offset = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => offset.checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or_else(|| ax_err_type!(InvalidInput))?;
        *offset = new_offset;
        Ok(new_offset)
    }
}

impl Drop for File {
    fn drop(&mut self) {
        unsafe {
            let attr = self.node.access_unchecked().get_attr().unwrap();
            match attr.file_type() {
                FileType::File => {
                    self.node.access_unchecked().release().ok();
                }
                FileType::Fifo => {
                    let (read, write) = (
                        self.node.access(Cap::READ).is_ok(),
                        self.node.access(Cap::WRITE).is_ok(),
                    );
                    self.node.access_unchecked().release_fifo(read, write).ok();
                }
                _ => {
                    self.node.access_unchecked().release().ok();
                }
            }
        }
    }
}

impl FileLike for File {
    fn path(&self) -> AbsPath {
        self.path.to_owned()
    }

    /// Reads the file at the current position. Returns the number of bytes
    /// read.
    ///
    /// After the read, the cursor will be advanced by the number of bytes read.
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.read(buf).map_err(LinuxError::from)
    }

    /// Writes the file at the current position. Returns the number of bytes
    /// written.
    ///
    /// After the write, the cursor will be advanced by the number of bytes
    /// written.
    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        self.write(buf).map_err(LinuxError::from)
    }

    fn flush(&self) -> LinuxResult {
        Ok(self.flush()?)
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

    fn set_flags(&self, flags: OpenFlags) -> LinuxResult {
        *self.flags.write() = flags;
        Ok(())
    }
}

impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        File::read(self, buf)
    }
}

impl Write for File {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        File::write(self, buf)
    }

    fn flush(&mut self) -> AxResult<()> {
        self.node.access(Cap::WRITE)?.fsync()
    }
}
