/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::Arc;
use axerrno::{AxError, AxResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use axio::PollState;
use log::warn;
use ringbuffer::RingBuffer;
use spinlock::SpinNoIrq;
use tty::ioctl::{FromPrimitive, IoctlCommand};
use tty::ldisc::{Ldisc, WinSize};
use tty::termios::Termios;

use super::ptmx::Ptmx;
use super::PTS_PTMX_INO;

/// Pseudoterminal master device controller
pub struct PtyMaster {
    /// Main pseudoterminal device (PTMX)
    ptmx: Arc<Ptmx>,
    /// Index identifier for this pseudoterminal instance
    idx: usize,
    /// Foreground process group ID
    fg_pgid: AtomicUsize,
    /// Output buffer with line discipline processing (master → slave)
    ///
    /// Writen by master and read by slave
    pub(in crate::pts) output: Arc<Ldisc>,
    /// Raw input buffer with spinlock protection (slave → master)
    ///
    /// Writen by slave and read by master
    pub(in crate::pts) input: SpinNoIrq<RingBuffer>,
}

impl PtyMaster {
    pub fn new(ptmx: Arc<Ptmx>, idx: usize) -> Self {
        Self {
            ptmx,
            idx,
            output: Ldisc::new(),
            input: SpinNoIrq::new(RingBuffer::new(ruxconfig::PTY_BUFFER_CAPACITY)),
            fg_pgid: AtomicUsize::new(1000),
        }
    }

    pub fn idx(&self) -> usize {
        self.idx
    }

    /// Retrieves current foreground process group ID
    pub fn fg_pgid(&self) -> usize {
        self.fg_pgid.load(Ordering::Acquire)
    }

    /// Updates foreground process group ID with release ordering
    pub fn set_fg_pgid(&self, fg_pgid: usize) {
        self.fg_pgid.store(fg_pgid, Ordering::Release);
    }
}

impl VfsNodeOps for PtyMaster {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            PTS_PTMX_INO,
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        self.ptmx.ptsfs().remove_pty(self.idx);
        Ok(())
    }

    fn read_at(&self, _offset: u64, dst: &mut [u8]) -> VfsResult<usize> {
        let mut input = self.input.lock();
        if input.is_empty() {
            return Err(AxError::WouldBlock);
        }
        Ok(input.read(dst))
    }

    fn write_at(&self, _offset: u64, src: &[u8]) -> VfsResult<usize> {
        let mut input = self.input.lock();
        for ch in src {
            self.output.push_char(*ch, |content| {
                input.write(content.as_bytes());
            });
        }
        Ok(src.len())
    }

    fn poll(&self) -> AxResult<PollState> {
        Ok(PollState {
            readable: !self.input.lock().is_empty(),
            writable: self.output.poll().writable,
            // TODO: pollhup
            pollhup: false,
        })
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> VfsResult<usize> {
        let ioctl_cmd = IoctlCommand::from_primitive(cmd as u32);
        log::debug!("Pty ioctl cmd: {:?}", ioctl_cmd);
        match ioctl_cmd {
            IoctlCommand::TCGETS | IoctlCommand::TCGETA => {
                let termios = self.output.termios();
                unsafe { *(arg as *mut Termios) = termios };
                Ok(0)
            }
            IoctlCommand::TCSETS | IoctlCommand::TCSETSW => {
                let termios = unsafe { *(arg as *const Termios) };
                self.output.set_termios(&termios);
                Ok(0)
            }
            IoctlCommand::TIOCSPTLCK => {
                // TODO
                Ok(0)
            }
            IoctlCommand::TIOCGPTPEER => {
                todo!();
            }
            IoctlCommand::TIOCGPTN => {
                unsafe { *(arg as *mut u32) = self.idx as _ };
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
                self.output.set_termios(&termios);
                self.output.clear_input();
                Ok(0)
            }
            IoctlCommand::TIOCGWINSZ => {
                let winsize = self.output.winsize();
                unsafe { *(arg as *mut WinSize) = winsize };
                Ok(0)
            }
            IoctlCommand::TIOCSWINSZ => {
                let winsize = unsafe { *(arg as *const WinSize) };
                self.output.set_winsize(&winsize);
                Ok(0)
            }
            _ => {
                warn!("unimplemented tty ioctl, cmd {:?} {:x}", ioctl_cmd, cmd);
                Ok(0)
            }
        }
    }

    fn fsync(&self) -> VfsResult {
        Ok(())
    }
}
