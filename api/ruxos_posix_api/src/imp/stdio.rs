/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axerrno::AxResult;
use axio::{prelude::*, BufReader};
use axsync::Mutex;

#[cfg(feature = "fd")]
use {
    alloc::sync::Arc,
    axerrno::{AxError, LinuxError, LinuxResult},
    axio::PollState,
    core::sync::atomic::{AtomicBool, Ordering},
};

struct StdinRaw;
struct StdoutRaw;

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
static STDIO_TTY_NAME: lazy_init::LazyInit<alloc::string::String> = lazy_init::LazyInit::new();
#[cfg(not(feature = "alloc"))]
static STDIO_TTY_NAME: &str = "dummy";

fn get_stdio_tty_name() -> &'static str {
    #[cfg(feature = "alloc")]
    {
        if !STDIO_TTY_NAME.is_init() {
            let name = ruxhal::get_all_device_names().first().unwrap().clone();
            STDIO_TTY_NAME.init_by(name);
        }
    }
    &STDIO_TTY_NAME
}

impl Read for StdinRaw {
    // Non-blocking read, returns number of bytes read.
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        Ok(ruxhal::tty_read(buf, get_stdio_tty_name()))
    }
}

impl Write for StdoutRaw {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        Ok(ruxhal::tty_write(buf, get_stdio_tty_name()))
    }

    fn flush(&mut self) -> AxResult {
        Ok(())
    }
}

pub struct Stdin {
    inner: &'static Mutex<BufReader<StdinRaw>>,
    #[cfg(feature = "fd")]
    nonblocking: AtomicBool,
}

impl Stdin {
    // Block until at least one byte is read.
    fn read_blocked(&self, buf: &mut [u8]) -> AxResult<usize> {
        let read_len = self.inner.lock().read(buf)?;
        if buf.is_empty() || read_len > 0 {
            return Ok(read_len);
        }
        // try again until we get something
        loop {
            let read_len = self.inner.lock().read(buf)?;
            if read_len > 0 {
                return Ok(read_len);
            }
            crate::sys_sched_yield();
        }
    }

    // Attempt a non-blocking read operation.
    #[cfg(feature = "fd")]
    fn read_nonblocked(&self, buf: &mut [u8]) -> AxResult<usize> {
        if let Some(mut inner) = self.inner.try_lock() {
            let read_len = inner.read(buf)?;
            Ok(read_len)
        } else {
            Err(AxError::WouldBlock)
        }
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        self.read_blocked(buf)
    }
}

pub struct Stdout {
    inner: &'static Mutex<StdoutRaw>,
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        self.inner.lock().write(buf)
    }

    fn flush(&mut self) -> AxResult {
        self.inner.lock().flush()
    }
}

/// Constructs a new handle to the standard input of the current process.
pub fn stdin() -> Stdin {
    static INSTANCE: Mutex<BufReader<StdinRaw>> = Mutex::new(BufReader::new(StdinRaw));
    Stdin {
        inner: &INSTANCE,
        #[cfg(feature = "fd")]
        nonblocking: AtomicBool::from(false),
    }
}

/// Constructs a new handle to the standard output of the current process.
pub fn stdout() -> Stdout {
    static INSTANCE: Mutex<StdoutRaw> = Mutex::new(StdoutRaw);
    Stdout { inner: &INSTANCE }
}

#[cfg(feature = "fd")]
impl ruxfdtable::FileLike for Stdin {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        match self.nonblocking.load(Ordering::Relaxed) {
            true => Ok(self.read_nonblocked(buf)?),
            false => Ok(self.read_blocked(buf)?),
        }
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<ruxfdtable::RuxStat> {
        let st_mode = 0o20000 | 0o440u32; // S_IFCHR | r--r-----
        Ok(ruxfdtable::RuxStat::from(crate::ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        }))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, nonblocking: bool) -> LinuxResult {
        self.nonblocking.store(nonblocking, Ordering::Relaxed);
        Ok(())
    }
}

#[cfg(feature = "fd")]
impl ruxfdtable::FileLike for Stdout {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<ruxfdtable::RuxStat> {
        let st_mode = 0o20000 | 0o220u32; // S_IFCHR | -w--w----
        Ok(ruxfdtable::RuxStat::from(crate::ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            ..Default::default()
        }))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}
