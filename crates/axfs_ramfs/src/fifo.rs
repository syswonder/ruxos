use alloc::sync::Arc;
use axerrno::{LinuxError, LinuxResult};
use axfs_vfs::VfsNodeRef;
use axfs_vfs::{impl_vfs_non_dir_default, VfsNodeAttr, VfsNodeOps, VfsResult};
use core::sync::atomic::{AtomicUsize, Ordering};
use log::debug;
use ringbuffer::RingBuffer;
use spin::Mutex;

/// A simple FIFO implementation.
pub struct Fifo {
    buffer: Arc<Mutex<RingBuffer>>,
    readers: AtomicUsize,
    writers: AtomicUsize,
}

impl Fifo {
    // create a new fifo
    pub fn new() -> Self {
        Self {
            buffer: Arc::new(Mutex::new(RingBuffer::new(1024))),
            readers: AtomicUsize::new(0),
            writers: AtomicUsize::new(0),
        }
    }

    // read data from fifo
    pub fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        debug!("read data from fifo");
        loop {
            let mut rb = self.buffer.lock();
            if rb.available_read() == 0 {
                if self.writers.load(Ordering::SeqCst) == 0 {
                    // when there is no writer and no data in the buffer, return EOF
                    return Ok(0);
                } else {
                    drop(rb);
                    sched_yield();
                    continue;
                }
            }
            // call the read() method of the ring buffer to copy the data to buf
            let bytes_read = rb.read(buf);
            return Ok(bytes_read);
        }
    }

    // write data to fifo
    pub fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        debug!("write data to fifo");
        loop {
            let mut rb = self.buffer.lock();
            if self.readers.load(Ordering::SeqCst) == 0 {
                // when there is no reader, return EPIPE
                return Err(LinuxError::EPIPE);
            }
            let bytes_written = rb.write(buf);
            if bytes_written > 0 {
                return Ok(bytes_written);
            }
            drop(rb);
            sched_yield();
        }
    }
}

/// A node representing a FIFO.
pub struct FifoNode {
    ino: u64,
    fifo: Fifo,
}

impl FifoNode {
    /// Create a new FIFO node.
    pub fn new(ino: u64) -> Self {
        Self {
            ino,
            fifo: Fifo::new(),
        }
    }
}

impl VfsNodeOps for FifoNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_fifo(self.ino))
    }

    // for fifo, offset is useless and ignored
    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        Ok(self.fifo.read(buf).unwrap_or(0))
    }

    // for fifo, offset is useless and ignored
    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(self.fifo.write(buf).unwrap_or(0))
    }

    // check if there are any readers
    fn fifo_has_readers(&self) -> bool {
        self.fifo.readers.load(Ordering::SeqCst) > 0
    }

    // open a fifo node
    fn open_fifo(
        &self,
        read: bool,
        write: bool,
        non_blocking: bool,
    ) -> VfsResult<Option<VfsNodeRef>> {
        debug!(
            "open a fifo node: read={}, write={}, non_blocking={}",
            read, write, non_blocking
        );
        if read {
            self.fifo.readers.fetch_add(1, Ordering::SeqCst);
            if !non_blocking {
                while self.fifo.writers.load(Ordering::SeqCst) == 0 {
                    sched_yield();
                }
            }
        }
        if write {
            self.fifo.writers.fetch_add(1, Ordering::SeqCst);
            if !non_blocking {
                while self.fifo.readers.load(Ordering::SeqCst) == 0 {
                    sched_yield();
                }
            }
        }
        Ok(None)
    }

    // release a fifo node
    fn release_fifo(&self, read: bool, write: bool) -> VfsResult {
        debug!("release a fifo node");
        if read {
            self.fifo.readers.fetch_sub(1, Ordering::SeqCst);
        }
        if write {
            self.fifo.writers.fetch_sub(1, Ordering::SeqCst);
        }
        Ok(())
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

// here, we cannot use ruxtask for possible cyclic package dependency.
// so we use the following code to simulate the behavior of ruxtask::sched_yield.
// actually, sched_yield() is similar to sys_sched_yield() in ruxos_posix_api,
// the only difference is that we deleted the `#[cfg(feature = "multitask")]` attribute.
fn sched_yield() {
    #[cfg(not(feature = "multitask"))]
    if cfg!(feature = "irq") {
        ruxhal::arch::wait_for_irqs();
    } else {
        core::hint::spin_loop();
    }
}
