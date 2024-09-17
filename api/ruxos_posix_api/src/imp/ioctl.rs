/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::sys_getpgid;
use axerrno::LinuxError;
use core::ffi::c_int;
use ruxtask::fs::get_file_like;

/// IOCTL oprations
pub const TCGETS: usize = 0x5401;
pub const TIOCGPGRP: usize = 0x540F;
pub const TIOCSPGRP: usize = 0x5410;
pub const TIOCGWINSZ: usize = 0x5413;
pub const FIONBIO: usize = 0x5421;
pub const FIOCLEX: usize = 0x5451;

#[derive(Clone, Copy, Default)]
pub struct ConsoleWinSize {
    pub ws_row: u16,
    pub ws_col: u16,
    pub ws_xpixel: u16,
    pub ws_ypixel: u16,
}

/// ioctl implementation,
/// currently only support fd = 1
pub fn sys_ioctl(fd: c_int, request: usize, data: usize) -> c_int {
    debug!("sys_ioctl <= fd: {}, request: {}", fd, request);
    syscall_body!(sys_ioctl, {
        match request {
            FIONBIO => {
                unsafe {
                    get_file_like(fd)?.set_nonblocking(*(data as *const i32) > 0)?;
                }
                Ok(0)
            }
            TIOCGWINSZ => {
                let winsize = data as *mut ConsoleWinSize;
                unsafe {
                    *winsize = ConsoleWinSize::default();
                }
                Ok(0)
            }
            TCGETS => {
                debug!("sys_ioctl: tty TCGETS");
                Ok(0)
            }
            TIOCSPGRP => {
                warn!("stdout pretend to be tty");
                Ok(0)
            }
            TIOCGPGRP => {
                warn!("stdout TIOCGPGRP, pretend to be have a tty process group.");
                unsafe {
                    *(data as *mut u32) = sys_getpgid(0) as _;
                }
                Ok(0)
            }
            FIOCLEX => Ok(0),
            _ => Err(LinuxError::EINVAL),
        }
    })
}
