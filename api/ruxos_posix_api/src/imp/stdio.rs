/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "fd")]
use {
    alloc::sync::Arc,
    axerrno::{LinuxError, LinuxResult},
    axio::PollState,
    core::sync::atomic::{AtomicBool, Ordering},
    ruxfs::AbsPath,
};

use axerrno::{AxError, AxResult};
use axio::prelude::*;

#[derive(Default)]
pub struct Stdin {
    #[cfg(feature = "fd")]
    nonblocking: AtomicBool,
}

pub struct Stdout;

impl Stdin {
    fn read_inner(&self, buf: &mut [u8]) -> AxResult<usize> {
        loop {
            #[cfg(not(all(feature = "irq", target_arch = "aarch64")))]
            {
                // Only the aarch64 architecture implements UART IRQ Handler, which asynchronously
                // transmits characters to Tty with higher efficiency. Current implementations for
                // x86_64 and riscv architectures lack this capability, requiring polling-based
                // reads from the console instead.
                while let Some(c) = ruxhal::console::getchar() {
                    ruxtty::tty_receive_char(c);
                }
            }
            match ruxtty::tty_read(buf) {
                Ok(len) => return Ok(len),
                Err(AxError::WouldBlock) => {
                    #[cfg(feature = "fd")]
                    if self.nonblocking.load(Ordering::Relaxed) {
                        return Err(AxError::WouldBlock);
                    }
                    crate::sys_sched_yield();
                }
                Err(_) => unreachable!(),
            };
        }
    }
}

impl Read for Stdin {
    fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        self.read_inner(buf)
    }
}

impl Write for Stdout {
    fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        ruxtty::tty_write(buf)
    }

    fn flush(&mut self) -> AxResult {
        Ok(())
    }
}

#[cfg(feature = "fd")]
impl ruxfdtable::FileLike for Stdin {
    fn path(&self) -> AbsPath {
        AbsPath::new("/dev/stdin")
    }

    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.read_inner(buf).map_err(|_| LinuxError::EAGAIN)
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
        #[cfg(not(all(feature = "irq", target_arch = "aarch64")))]
        {
            while let Some(c) = ruxhal::console::getchar() {
                ruxtty::tty_receive_char(c);
            }
        }
        Ok(ruxtty::tty_poll())
    }

    fn set_nonblocking(&self, nonblocking: bool) -> LinuxResult {
        self.nonblocking.store(nonblocking, Ordering::Relaxed);
        Ok(())
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> LinuxResult<usize> {
        ruxtty::tty_ioctl(cmd, arg).map_err(LinuxError::from)
    }
}

#[cfg(feature = "fd")]
impl ruxfdtable::FileLike for Stdout {
    fn path(&self) -> AbsPath {
        AbsPath::new("/dev/stdout")
    }

    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EPERM)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        ruxtty::tty_write(buf).map_err(LinuxError::from)
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
            pollhup: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> LinuxResult<usize> {
        ruxtty::tty_ioctl(cmd, arg).map_err(LinuxError::from)
    }
}
