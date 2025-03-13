/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
#![allow(dead_code)]
use core::sync::atomic::{AtomicUsize, Ordering};

use crate::{
    ioctl::IoctlCommand,
    ldisc::{Ldisc, WinSize},
    termios::Termios,
};
use alloc::{ffi::CString, sync::Arc};
use axerrno::AxResult;
use axlog::{ax_print, warn};
use num_enum::FromPrimitive;

/// Terminal device representation with line discipline and process control
///
/// Manages character input/output processing and foreground process group
pub struct Tty {
    /// Device name
    name: CString,
    /// Line discipline handling input processing
    pub ldisc: Arc<Ldisc>,
    // TODO: Implement process group management
    fg_pgid: AtomicUsize,
}

impl Tty {
    /// Creates new TTY device
    pub fn new(name: CString) -> Arc<Self> {
        Arc::new(Self {
            name,
            ldisc: Ldisc::new(),
            fg_pgid: AtomicUsize::new(1000),
        })
    }

    // Handles incoming character with line discipline processing
    ///
    /// Invokes callback for immediate echo when applicable
    pub fn push_char(&self, ch: u8) {
        self.ldisc.push_char(ch, |content| ax_print!("{}", content));
    }

    /// Retrieves current foreground process group ID
    pub fn fg_pgid(&self) -> usize {
        self.fg_pgid.load(Ordering::Relaxed)
    }

    /// Updates foreground process group ID with release ordering
    pub fn set_fg_pgid(&self, fg_pgid: usize) {
        self.fg_pgid.store(fg_pgid, Ordering::Release);
    }

    pub fn ioctl(&self, cmd: usize, arg: usize) -> AxResult<usize> {
        match IoctlCommand::from_primitive(cmd as u32) {
            IoctlCommand::TCGETS | IoctlCommand::TCGETA => {
                let termios = self.ldisc.termios();
                unsafe { *(arg as *mut Termios) = termios };
                Ok(0)
            }
            IoctlCommand::TCSETS | IoctlCommand::TCSETSW => {
                let termios = unsafe { *(arg as *const Termios) };
                self.ldisc.set_termios(&termios);
                Ok(0)
            }
            IoctlCommand::TIOCGPGRP => {
                let fg_pgid = self.fg_pgid() as u32;
                unsafe {
                    *(arg as *mut u32) = fg_pgid;
                }
                Ok(0)
            }
            IoctlCommand::TIOCSPGRP => {
                let fg_pgid = unsafe { *(arg as *const u32) } as usize;
                self.set_fg_pgid(fg_pgid);
                Ok(0)
            }

            IoctlCommand::TCSETSF => {
                let termios = unsafe { *(arg as *const Termios) };
                self.ldisc.set_termios(&termios);
                self.ldisc.clear_input();
                Ok(0)
            }
            IoctlCommand::TIOCGWINSZ => {
                let winsize = self.ldisc.winsize();
                unsafe { *(arg as *mut WinSize) = winsize };
                Ok(0)
            }
            IoctlCommand::TIOCSWINSZ => {
                let winsize = unsafe { *(arg as *const WinSize) };
                self.ldisc.set_winsize(&winsize);
                Ok(0)
            }
            IoctlCommand::TIOCSCTTY => {
                warn!("TtyIoctlCmd::TIOCSCTTY not implemented");
                Ok(0)
            }
            _ => {
                warn!("unimplemented tty ioctl, cmd {cmd}");
                Ok(0)
            }
        }
    }
}
