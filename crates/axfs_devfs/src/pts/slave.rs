/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use core::sync::atomic::{AtomicUsize, Ordering};

use alloc::sync::{Arc, Weak};
use axerrno::AxResult;
use axfs_vfs::{
    impl_vfs_non_dir_default, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType,
    VfsResult,
};
use axio::PollState;
use log::warn;
use tty::ioctl::{FromPrimitive, IoctlCommand};

use super::{master::PtyMaster, PTS_PTMX_INO};

pub struct PtySlaveInode {
    slave: Arc<PtySlave>,
}

impl PtySlaveInode {
    pub fn new(slave: Arc<PtySlave>) -> Self {
        Self { slave }
    }
}

impl VfsNodeOps for PtySlaveInode {
    fn open(&self) -> VfsResult<Option<VfsNodeRef>> {
        Ok(Some(self.slave.clone()))
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            self.slave.idx() as u64 + PTS_PTMX_INO + 1,
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        Ok(())
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.slave.read_at(offset, buf)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.slave.write_at(offset, buf)
    }
    impl_vfs_non_dir_default!();
}

pub struct PtySlave {
    master: Weak<PtyMaster>,
    fg_pgid: AtomicUsize,
}

impl PtySlave {
    pub fn new(master: &Arc<PtyMaster>) -> Self {
        Self {
            master: Arc::downgrade(master),
            fg_pgid: AtomicUsize::new(1000),
        }
    }

    pub fn master(&self) -> Arc<PtyMaster> {
        self.master.upgrade().unwrap()
    }

    fn fg_pgid(&self) -> usize {
        self.fg_pgid.load(Ordering::Acquire)
    }

    pub fn set_fg_pgid(&self, fg_pgid: usize) {
        self.fg_pgid.store(fg_pgid, Ordering::Release);
    }

    pub fn idx(&self) -> usize {
        self.master().idx()
    }
}

impl VfsNodeOps for PtySlave {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            self.idx() as u64 + PTS_PTMX_INO + 1,
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        Ok(())
    }

    fn read_at(&self, _offset: u64, dst: &mut [u8]) -> VfsResult<usize> {
        self.master().output.read(dst)
    }

    fn write_at(&self, _offset: u64, src: &[u8]) -> VfsResult<usize> {
        let master = self.master();
        let mut buf = master.input.lock();
        for ch in src {
            if *ch == b'\n' {
                buf.force_enqueue(b'\r');
                buf.force_enqueue(b'\n');
            } else {
                buf.force_enqueue(*ch);
            }
        }
        Ok(src.len())
    }

    fn ioctl(&self, cmd: usize, arg: usize) -> VfsResult<usize> {
        let ioctl_cmd = IoctlCommand::from_primitive(cmd as u32);
        match ioctl_cmd {
            IoctlCommand::TCGETS
            | IoctlCommand::TCSETS
            | IoctlCommand::TIOCGPTN
            | IoctlCommand::TIOCGWINSZ
            | IoctlCommand::TIOCSWINSZ => self.master().ioctl(cmd, arg),

            IoctlCommand::FIONREAD => {
                let len = self.master().output.read_buffer_len();
                unsafe { *(arg as *mut u32) = len as u32 };
                Ok(0)
            }
            IoctlCommand::TIOCGPGRP => {
                let fg_pgid = self.fg_pgid() as u32;
                unsafe {
                    *(arg as *mut u32) = fg_pgid;
                };
                Ok(0)
            }
            IoctlCommand::TIOCSPGRP => {
                let fg_pgid = unsafe { *(arg as *const u32) } as usize;
                self.set_fg_pgid(fg_pgid);
                Ok(0)
            }
            _ => {
                warn!("unimplemented tty ioctl, cmd {ioctl_cmd:?} {cmd:x}");
                Ok(0)
            }
        }
    }

    fn poll(&self) -> AxResult<PollState> {
        let master = self.master();
        let readable = master.output.poll().readable;
        let writable = master.input.lock().is_full();
        Ok(PollState {
            readable,
            writable,
            pollhup: false,
        })
    }

    fn fsync(&self) -> VfsResult {
        Ok(())
    }
}
