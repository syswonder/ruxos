/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use crate::alloc::string::ToString;
use crate::dir::split_path;
use axerrno::ax_err;
use axfs_vfs::{
    impl_vfs_dir_default, RelPath, VfsDirEntry, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm,
    VfsNodeRef, VfsNodeType, VfsResult,
};
use spin::once::Once;
use spin::rwlock::RwLock;

use super::ptmx::Ptmx;
use super::slave::PtySlaveInode;
use super::{PtsFileSystem, PTS_ROOT_INO};
use alloc::string::String;
use alloc::sync::{Arc, Weak};
use alloc::vec::Vec;

type PtyIndexStr = String;

/// The root node in pts file system
pub struct PtsRootInode {
    parent: Once<Weak<dyn VfsNodeOps>>,
    ptmx: Arc<Ptmx>,
    slave_inodes: RwLock<Vec<(PtyIndexStr, Arc<PtySlaveInode>)>>,
}

impl PtsRootInode {
    pub fn new(fs: Weak<PtsFileSystem>) -> Arc<Self> {
        Arc::new(Self {
            parent: Once::new(),
            ptmx: Ptmx::new(fs),
            slave_inodes: RwLock::new(Vec::new()),
        })
    }

    pub fn set_parent(&self, parent: &Arc<dyn VfsNodeOps>) {
        self.parent.call_once(|| Arc::downgrade(parent));
    }

    pub fn ptmx(&self) -> &Arc<Ptmx> {
        &self.ptmx
    }

    pub fn push_slave(&self, idx: usize, slave: Arc<PtySlaveInode>) {
        self.slave_inodes.write().push((idx.to_string(), slave));
    }

    pub fn remove_slave(&self, idx: usize) {
        let mut slave_inodes = self.slave_inodes.write();
        if let Some(pos) = slave_inodes.iter().position(|(s, _)| s == &idx.to_string()) {
            slave_inodes.remove(pos);
        }
    }
}

impl VfsNodeOps for PtsRootInode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            PTS_ROOT_INO,
            VfsNodePerm::default_file(),
            VfsNodeType::Dir,
            0,
            0,
        ))
    }

    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        Ok(())
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.parent.get().unwrap().upgrade()
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                ".." => self.parent().ok_or(VfsError::NotFound)?.lookup(&rest),
                _ => Err(VfsError::NotFound), // there is no sub directory
            }
        } else if name.is_empty() {
            Ok(self.clone() as VfsNodeRef)
        } else if name == ".." {
            self.parent().ok_or(VfsError::NotFound)
        } else if name == "ptmx" {
            Ok(self.ptmx.clone())
        } else {
            for (pty_index, slave_inode) in self.slave_inodes.read().iter() {
                if name == pty_index {
                    return Ok(slave_inode.clone());
                }
            }
            Err(VfsError::NotFound)
        }
    }

    fn read_dir(
        &self,
        start_idx: usize,
        dirents: &mut [axfs_vfs::VfsDirEntry],
    ) -> VfsResult<usize> {
        let slave_inodes = self.slave_inodes.read();
        let mut slave_iter = slave_inodes.iter().skip(start_idx.max(3) - 3);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                2 => *ent = VfsDirEntry::new("ptmx", VfsNodeType::CharDevice),
                _ => {
                    if let Some((name, _)) = slave_iter.next() {
                        *ent = VfsDirEntry::new(name, VfsNodeType::CharDevice);
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    fn create(&self, _path: &RelPath, _ty: axfs_vfs::VfsNodeType, _mode: VfsNodePerm) -> VfsResult {
        ax_err!(PermissionDenied)
    }

    fn link(&self, _name: &RelPath, _src: Arc<dyn VfsNodeOps>) -> VfsResult<Arc<dyn VfsNodeOps>> {
        ax_err!(PermissionDenied)
    }

    fn unlink(&self, _path: &RelPath) -> VfsResult {
        ax_err!(PermissionDenied)
    }

    fn rename(&self, _src_path: &RelPath, _dst_path: &RelPath) -> VfsResult<()> {
        ax_err!(PermissionDenied)
    }

    impl_vfs_dir_default!();
}
