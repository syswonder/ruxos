/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use super::{PtsFileSystem, PTS_PTMX_INO};
use alloc::sync::{Arc, Weak};
use axfs_vfs::{
    impl_vfs_non_dir_default, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType,
    VfsResult,
};

/// Pseudo-Terminal Master Multiplexer
pub struct Ptmx {
    ptsfs: Weak<PtsFileSystem>,
}

impl Ptmx {
    pub fn new(fs: Weak<PtsFileSystem>) -> Arc<Self> {
        Arc::new(Self { ptsfs: fs })
    }

    pub fn ptsfs(&self) -> Arc<PtsFileSystem> {
        self.ptsfs.upgrade().unwrap()
    }
}

impl VfsNodeOps for Ptmx {
    fn open(&self) -> VfsResult<Option<VfsNodeRef>> {
        let ptsfs = self.ptsfs.upgrade().unwrap();
        Ok(Some(ptsfs.allocate_pty()))
    }

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

    impl_vfs_non_dir_default!();
}
