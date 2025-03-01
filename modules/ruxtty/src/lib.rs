/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Portions of this code are inspired by the design of Asterinas OS

#![no_std]
extern crate alloc;

mod driver;
mod ioctl;
mod ldisc;
mod termios;
mod tty;

use alloc::{ffi::CString, sync::Arc};
use axerrno::AxResult;
use axio::PollState;
use axlog::{ax_print, ax_println};
use driver::TtyDriver;
use lazy_init::LazyInit;
use spin::once::Once;
use tty::Tty;

/// Global singleton instance for the default TTY device
static N_TTY: Once<Arc<Tty>> = Once::new();

/// all tty drivers.
/// only be written when registering a driver.
///
/// Current now there is only ttyS device
static TTY_DRIVER: LazyInit<Arc<TtyDriver>> = LazyInit::new();

/// Initializes the TTY subsystem, creates a new TTY device and registers it
pub fn init_tty() {
    let tty = Tty::new(CString::new("ttyS").unwrap());
    N_TTY.call_once(|| tty.clone());
    let driver = TtyDriver::new();
    driver.add_tty(tty);
    TTY_DRIVER.init_by(driver);
}

/// Pushes received data into the TTY driver's input buffer
/// # Note: This implementation is ​**only enabled for `aarch64` architecture** since others haven't implement irq
pub fn tty_receive_buf(buf: &[u8]) {
    TTY_DRIVER.try_get().unwrap().push_slice(buf);
}

/// Pushes received char into the TTY driver's input buffer
pub fn tty_receive_char(ch: u8) {
    TTY_DRIVER.try_get().unwrap().push_char(ch);
}

/// Reads data from TTY line discipline into the destination buffer
pub fn tty_read(dst: &mut [u8]) -> AxResult<usize> {
    N_TTY.get().unwrap().ldisc.read(dst)
}

/// Checks if TTY has data available for reading
pub fn tty_poll() -> PollState {
    N_TTY.get().unwrap().ldisc.poll()
}

/// Writes data to TTY output, handles UTF-8 and binary content
pub fn tty_write(src: &[u8]) -> AxResult<usize> {
    if let Ok(content) = alloc::str::from_utf8(src) {
        ax_print!("{}", content);
    } else {
        ax_println!("Not utf-8 content: {:?}", src);
    }
    Ok(src.len())
}

/// Handles TTY-specific ioctl commands
pub fn tty_ioctl(cmd: usize, arg: usize) -> AxResult<usize> {
    N_TTY.get().unwrap().ioctl(cmd, arg)
}
