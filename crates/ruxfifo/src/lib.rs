/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Fifo(named pipe) used by RuxOS
#![no_std]
extern crate alloc;
use alloc::sync::Arc;
use axerrno::AxError;
use axfs_vfs::{impl_vfs_non_dir_default, VfsNodeAttr, VfsNodeOps, VfsResult};
use axfs_vfs::{VfsNodePerm, VfsNodeType};
use axio::PollState;
use core::sync::atomic::{AtomicUsize, Ordering};
use ringbuffer::RingBuffer;
use spin::mutex::Mutex;

/// Default size of fifo
const FIFO_SIZE: usize = 65536;

/// FIFO (named pipe) node implementation for inter-process communication
///
/// Provides synchronized read/write operations through a fixed-size ring buffer,
/// with atomic reference counting for concurrent access management.
pub struct FifoNode {
    /// VFS node attributes
    attr: VfsNodeAttr,
    /// Thread-safe ring buffer with mutual exclusion lock
    buffer: Mutex<RingBuffer>,
    /// Active readers counter (atomic for lock-free access)
    readers: AtomicUsize,
    /// Active writers counter (atomic for lock-free access)
    writers: AtomicUsize,
}

impl FifoNode {
    /// create a new fifo
    pub fn new(ino: u64) -> Self {
        Self {
            attr: VfsNodeAttr::new(ino, VfsNodePerm::default_fifo(), VfsNodeType::Fifo, 0, 0),
            buffer: Mutex::new(RingBuffer::new(FIFO_SIZE)),
            readers: AtomicUsize::new(0),
            writers: AtomicUsize::new(0),
        }
    }

    /// Creates an interconnected pair of FIFO nodes for pipe communication
    ///
    /// Returns two `Arc` references sharing the same underlying buffer and counters,
    /// typically used for creating pipe reader/writer endpoints.
    pub fn new_pair() -> (Arc<Self>, Arc<Self>) {
        let node = Arc::new(Self {
            attr: VfsNodeAttr::new(1, VfsNodePerm::default_fifo(), VfsNodeType::Fifo, 0, 0),
            buffer: Mutex::new(RingBuffer::new(FIFO_SIZE)),
            readers: AtomicUsize::new(1),
            writers: AtomicUsize::new(1),
        });
        (node.clone(), node)
    }

    /// Returns current number of active readers
    pub fn readers(&self) -> usize {
        self.readers.load(Ordering::Acquire)
    }

    /// Registers a new reader with atomic reference counting
    ///
    /// # Memory Ordering
    /// Uses `Ordering::AcqRel` to synchronize with other atomic operations:
    /// - Acquire: See previous writes to the buffer
    /// - Release: Make buffer writes visible to others
    pub fn acquire_reader(&self) {
        self.readers.fetch_add(1, Ordering::AcqRel);
    }

    /// Unregisters a reader and checks for underflow
    pub fn release_reader(&self) {
        let cnt_before = self.readers.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(cnt_before != 0)
    }

    /// Returns current number of active writers
    pub fn writers(&self) -> usize {
        self.writers.load(Ordering::Acquire)
    }

    /// Registers a new writer with atomic reference counting
    pub fn acquire_writer(&self) {
        self.writers.fetch_add(1, Ordering::AcqRel);
    }

    /// Unregisters a writer and checks for underflow
    pub fn release_writer(&self) {
        let cnt_before = self.writers.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(cnt_before != 0)
    }

    /// Checks readable status and peer existence for readers
    pub fn reader_poll(&self) -> VfsResult<PollState> {
        let buffer = self.buffer.lock();
        Ok(PollState {
            readable: !buffer.is_empty(),
            writable: false,
            pollhup: self.writers() == 0,
        })
    }

    /// Checks writable status and peer existence for writers
    pub fn writer_poll(&self) -> VfsResult<PollState> {
        let buffer = self.buffer.lock();
        Ok(PollState {
            readable: false,
            writable: !buffer.is_full(),
            pollhup: self.readers() == 0,
        })
    }
}

impl VfsNodeOps for FifoNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(self.attr)
    }

    // for fifo, offset is useless and ignored
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut buffer = self.buffer.lock();
        if buffer.is_empty() {
            return Err(AxError::WouldBlock);
        }
        Ok(buffer.read(buf))
    }

    // for fifo, offset is useless and ignored
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut buffer = self.buffer.lock();
        if buffer.is_full() {
            return Err(AxError::WouldBlock);
        }
        Ok(buffer.write(buf))
    }

    // fifo does not support truncate
    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    // fifo does not support fsync
    fn fsync(&self) -> VfsResult {
        Ok(())
    }

    impl_vfs_non_dir_default! {}
}
