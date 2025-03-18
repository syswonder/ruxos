/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::sync::{Arc, Weak};
use core::ffi::c_int;
use ringbuffer::RingBuffer;
use ruxfs::AbsPath;

use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use axsync::Mutex;
use ruxfdtable::{FileLike, OpenFlags, RuxStat};

use crate::{ctypes, sys_fcntl};
use ruxtask::fs::{add_file_like, close_file_like};

pub struct Pipe {
    readable: bool,
    buffer: Arc<Mutex<RingBuffer>>,
    // to find the write end when the read end is closed
    _write_end_closed: Option<Weak<Mutex<RingBuffer>>>,
}

impl Pipe {
    pub fn new() -> (Pipe, Pipe) {
        let buffer = Arc::new(Mutex::new(RingBuffer::new(ruxconfig::PIPE_BUFFER_SIZE)));
        let read_end = Pipe {
            readable: true,
            buffer: buffer.clone(),
            _write_end_closed: None,
        };
        let write_end = Pipe {
            readable: false,
            buffer: buffer.clone(),
            _write_end_closed: Some(Arc::downgrade(&buffer)),
        };
        (read_end, write_end)
    }

    pub const fn readable(&self) -> bool {
        self.readable
    }

    pub const fn writable(&self) -> bool {
        !self.readable
    }

    pub fn write_end_close(&self) -> bool {
        let write_end_count = Arc::weak_count(&self.buffer);
        write_end_count == 0
    }
}

impl FileLike for Pipe {
    fn path(&self) -> AbsPath {
        AbsPath::new("/dev/pipe")
    }

    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        if !self.readable() {
            return Err(LinuxError::EPERM);
        }
        let mut read_size = 0usize;
        let max_len = buf.len();
        let mut ring_buffer = self.buffer.lock();
        // First, check if there is data in the read end.
        // This loop is only runs when the write end is open
        // and there is no data available
        loop {
            let loop_read = ring_buffer.available_read();
            // If there is no data
            if loop_read == 0 {
                if self.write_end_close() {
                    // write end is closed, read 0 bytes.
                    return Ok(0);
                } else {
                    // write end is open
                    drop(ring_buffer);
                    // Data not ready, wait for write end
                    crate::sys_sched_yield(); // TODO: use synconize primitive
                    ring_buffer = self.buffer.lock();
                }
            } else {
                break;
            }
        }
        // read data
        let loop_read = ring_buffer.available_read();
        for _ in 0..loop_read {
            if read_size == max_len {
                return Ok(read_size);
            }
            debug_assert!(!ring_buffer.is_empty());
            buf[read_size] = ring_buffer.dequeue().unwrap();
            read_size += 1;
        }
        Ok(read_size)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        if !self.writable() {
            return Err(LinuxError::EPERM);
        }
        let mut write_size = 0usize;
        let max_len = buf.len();
        loop {
            let mut ring_buffer = self.buffer.lock();
            let loop_write = ring_buffer.available_write();
            if loop_write == 0 {
                drop(ring_buffer);
                // Buffer is full, wait for read end to consume
                crate::sys_sched_yield(); // TODO: use synconize primitive
                continue;
            }
            for _ in 0..loop_write {
                if write_size == max_len {
                    return Ok(write_size);
                }
                debug_assert!(!ring_buffer.is_full());
                ring_buffer.enqueue(buf[write_size]);
                write_size += 1;
            }
        }
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let st_mode = 0o10000 | 0o600u32; // S_IFIFO | rw-------
        Ok(RuxStat::from(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_blksize: 4096,
            ..Default::default()
        }))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        let buf = self.buffer.lock();
        Ok(PollState {
            readable: self.readable() && buf.available_read() > 0,
            writable: self.writable() && buf.available_write() > 0,
            pollhup: self.write_end_close(),
        })
    }

    fn set_flags(&self, _flags: OpenFlags) -> LinuxResult {
        Ok(())
    }
}

/// Create a pipe
///
/// Return 0 if succeed
pub fn sys_pipe(fds: &mut [c_int]) -> c_int {
    debug!("sys_pipe <= {:#x}", fds.as_ptr() as usize);
    syscall_body!(sys_pipe, {
        if fds.len() != 2 {
            return Err(LinuxError::EFAULT);
        }

        let (read_end, write_end) = Pipe::new();
        let read_fd = add_file_like(Arc::new(read_end), OpenFlags::empty())?;
        let write_fd =
            add_file_like(Arc::new(write_end), OpenFlags::empty()).inspect_err(|_| {
                close_file_like(read_fd).ok();
            })?;

        fds[0] = read_fd as c_int;
        fds[1] = write_fd as c_int;

        debug!("[sys_pipe] create pipe with read fd {read_fd} and write fd {write_fd}");
        Ok(0)
    })
}

/// `pipe2` syscall, used by AARCH64
///
/// Return 0 on success
pub fn sys_pipe2(fds: &mut [c_int], flag: c_int) -> c_int {
    debug!(
        "sys_pipe2 <= fds: {:#x}, flag: {}",
        fds.as_ptr() as usize,
        flag
    );
    let ret = sys_pipe(fds);
    if ret < 0 {
        return ret;
    }
    syscall_body!(sys_pipe2, {
        if (flag as u32 & !(ctypes::O_CLOEXEC | ctypes::O_NONBLOCK)) != 0 {
            return Err(LinuxError::EINVAL);
        }

        if (flag as u32 & ctypes::O_CLOEXEC) != 0 {
            sys_fcntl(fds[0], ctypes::F_SETFD as _, ctypes::FD_CLOEXEC as _);
            sys_fcntl(fds[1], ctypes::F_SETFD as _, ctypes::FD_CLOEXEC as _);
        }

        if (flag as u32 & ctypes::O_NONBLOCK) != 0 {
            sys_fcntl(fds[0], ctypes::F_SETFL as _, ctypes::O_NONBLOCK as _);
            sys_fcntl(fds[1], ctypes::F_SETFL as _, ctypes::O_NONBLOCK as _);
        }
        Ok(0)
    })
}
