/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Named pipe (FIFO) implementation for VFS
use crate::AbsPath;
use alloc::sync::Arc;
use axerrno::{AxError, LinuxError, LinuxResult};
use axfs_vfs::VfsNodeOps;
use axio::PollState;
use crate_interface::call_interface;
use ruxfdtable::{FileLike, OpenFlags, RuxStat};
use ruxfifo::FifoNode;
use spin::rwlock::RwLock;

/// Reader endpoint for both FIFO (named pipe) and Pipe communication
pub struct FifoReader {
    /// Absolute path in virtual filesystem
    path: AbsPath<'static>,
    /// Shared FIFO buffer
    node: Arc<FifoNode>,
    /// Current open flags, e.g. `O_NONBLOCK`
    flags: RwLock<OpenFlags>,
}

impl FifoReader {
    /// Opening a FIFO for reading data (using the open() function with the O_RDONLY flag)
    /// block until another process opens the FIFO for writing data (using the open() function with the O_WRONLY flag).
    pub fn new(path: AbsPath<'static>, node: Arc<FifoNode>, flags: OpenFlags) -> Self {
        node.acquire_reader();
        while node.writers() == 0 {
            // PERF: use wait and wakeup instead of yield now
            call_interface!(SchedYieldIf::yield_now);
            if flags.contains(OpenFlags::O_NONBLOCK) {
                // Opening a FIFO for reading is safe when the other end has no writer, as read operations will return no data.
                break;
            }
        }
        Self {
            path,
            node,
            flags: RwLock::new(flags),
        }
    }
}

impl Drop for FifoReader {
    /// Decrements reader count and triggers wakeups
    fn drop(&mut self) {
        call_interface!(SchedYieldIf::yield_now);
        self.node.release_reader()
    }
}

impl FileLike for FifoReader {
    fn path(&self) -> AbsPath {
        self.path.to_owned()
    }

    /// Reads data from FIFO with blocking behavior
    ///
    /// - Returns 0 when all writers closed and buffer empty
    /// - EAGAIN if non-blocking and no data available
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        loop {
            match self.node.read_at(0, buf) {
                Ok(len) => return Ok(len),
                Err(AxError::WouldBlock) => {
                    // writer end closed and there is no data to read
                    if self.node.writers() == 0 {
                        return Ok(0);
                    }
                    if self.flags.read().contains(OpenFlags::O_NONBLOCK) {
                        return Err(LinuxError::EAGAIN);
                    }
                    crate_interface::call_interface!(SchedYieldIf::yield_now);
                }
                err => return err.map_err(LinuxError::from),
            }
        }
    }

    /// fd is not open for writing. In this case should return `LinuxError::EBADF`
    /// See `<https://man7.org/linux/man-pages/man2/write.2.html>`
    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EBADF)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        Ok(RuxStat::from(self.node.get_attr()?))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        self.node.reader_poll().map_err(LinuxError::from)
    }

    fn set_flags(&self, flags: OpenFlags) -> LinuxResult {
        *self.flags.write() = flags;
        Ok(())
    }

    fn flags(&self) -> OpenFlags {
        *self.flags.read() | OpenFlags::O_RDONLY
    }
}

/// Writer endpoint for both FIFO (named pipe) and Pipe communication
pub struct FifoWriter {
    path: AbsPath<'static>,
    node: Arc<FifoNode>,
    flags: RwLock<OpenFlags>,
}

impl FifoWriter {
    /// Creates new writer endpoint with POSIX error handling
    ///
    /// # Error Conditions
    /// - `ENXIO` when non-blocking and no readers available
    /// - Blocks indefinitely without O_NONBLOCK until reader appears
    pub fn new(path: AbsPath<'static>, node: Arc<FifoNode>, flags: OpenFlags) -> LinuxResult<Self> {
        node.acquire_writer();
        while node.readers() == 0 {
            // PERF: use wait and wakeup instead of yield now
            call_interface!(SchedYieldIf::yield_now);
            if flags.contains(OpenFlags::O_NONBLOCK) {
                // opening a FIFO with ​**O_WRONLY** and ​**O_NONBLOCK** flags if no process has the FIFO open for reading will cause err
                return Err(LinuxError::ENXIO);
            }
        }
        Ok(Self {
            path,
            node,
            flags: RwLock::new(flags),
        })
    }
}

impl Drop for FifoWriter {
    fn drop(&mut self) {
        call_interface!(SchedYieldIf::yield_now);
        self.node.release_writer();
    }
}

impl FileLike for FifoWriter {
    fn path(&self) -> AbsPath {
        self.path.to_owned()
    }

    /// fd is not open for reading. In this case should return `LinuxError::EBADF`
    /// See `<https://man7.org/linux/man-pages/man2/read.2.html>`
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EBADF)
    }

    /// Writes data with backpressure management
    ///
    /// # Error Conditions
    /// - `EPIPE` when all readers closed
    /// - `EAGAIN` if non-blocking and buffer full
    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        // TODO: when FifoNode has no reader when writing, send `SIGPIPE` signal
        if self.node.readers() == 0 {
            return Err(LinuxError::EPIPE);
        }
        loop {
            match self.node.write_at(0, buf) {
                Ok(len) => return Ok(len),
                Err(AxError::WouldBlock) => {
                    if self.flags.read().contains(OpenFlags::O_NONBLOCK) {
                        return Err(LinuxError::EAGAIN);
                    }
                    crate_interface::call_interface!(SchedYieldIf::yield_now);
                }
                Err(_) => todo!(),
            }
        }
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        Ok(RuxStat::from(self.node.get_attr()?))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        self.node.writer_poll().map_err(LinuxError::from)
    }

    fn set_flags(&self, flags: OpenFlags) -> LinuxResult {
        *self.flags.write() = flags;
        Ok(())
    }

    fn flags(&self) -> OpenFlags {
        *self.flags.read() | OpenFlags::O_WRONLY
    }
}

/// Creates connected pipe pair with shared buffer
///
/// If no endpoint is closed, readers count and writers count must always be 1 in `Arc<FifoNode>`,
/// because Arc clone won't increase the count
pub fn new_pipe_pair(flags: OpenFlags) -> (Arc<FifoReader>, Arc<FifoWriter>) {
    let (reader_node, writer_node) = FifoNode::new_pair();
    let reader = Arc::new(FifoReader {
        path: AbsPath::new(""),
        node: reader_node,
        flags: RwLock::new(flags),
    });
    let writer = Arc::new(FifoWriter {
        path: AbsPath::new(""),
        node: writer_node,
        flags: RwLock::new(flags),
    });
    (reader, writer)
}

#[crate_interface::def_interface]
/// task yield interface
pub trait SchedYieldIf {
    /// Yields CPU using appropriate scheduling strategy
    fn yield_now();
}
