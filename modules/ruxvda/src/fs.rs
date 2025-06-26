/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! VDA filesystem used by [Ruxos](https://github.com/syswonder/ruxos).
//!
//! The implementation is based on [`axfs_vfs`].

#![allow(dead_code)]

use alloc::vec;
use alloc::{sync::Arc, sync::Weak};
use axfs_vfs::{
    RelPath, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsOps,
    VfsResult,
};
use log::*;
use ruxdriver::prelude::BlockDriverOps;
use ruxdriver::AxBlockDevice;
use spin::{once::Once, RwLock};

const BLOCK_SIZE: usize = 512;

/// A VDA filesystem that implements [`axfs_vfs::VfsOps`].
pub struct VdaFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<VdaNode>,
}

impl VdaFileSystem {
    /// Create a new VDA filesystem.
    pub fn new(dev: AxBlockDevice) -> Self {
        info!("Create VDA filesystem");
        Self {
            parent: Once::new(),
            root: VdaNode::new(None, dev),
        }
    }
}

impl VfsOps for VdaFileSystem {
    fn mount(&self, parent: VfsNodeRef) -> VfsResult {
        self.root.set_parent(Some(self.parent.call_once(|| parent)));
        Ok(())
    }

    fn root_dir(&self) -> VfsNodeRef {
        debug!("Get root directory of VDA filesystem");
        self.root.clone()
    }
}

/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct VdaNode {
    this: Weak<VdaNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    transport: Arc<RwLock<AxBlockDevice>>,
}

impl VdaNode {
    pub(super) fn new(parent: Option<Weak<dyn VfsNodeOps>>, dev: AxBlockDevice) -> Arc<Self> {
        debug!("Create VDA node");
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            transport: Arc::new(RwLock::new(dev)),
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }
}

impl VfsNodeOps for VdaNode {
    fn open(&self) -> VfsResult<Option<VfsNodeRef>> {
        debug!("Open VDA node");
        Ok(None)
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        debug!("Lookup VDA node {:?}", path);
        Ok(self)
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        debug!("Get VDA node attributes");
        Ok(VfsNodeAttr::new(
            0,
            VfsNodePerm::from_bits_truncate(0o777),
            VfsNodeType::BlockDevice,
            67108864,
            131072,
        ))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        debug!(
            "Read VDA node offset: {:?}, % = {:?}, buf_len: {:?}, last: {:?}",
            offset,
            offset % BLOCK_SIZE as u64,
            buf.len(),
            (offset + buf.len() as u64) % BLOCK_SIZE as u64
        );

        let mut dev = self.transport.write();
        let mut cur_offset = offset;
        let mut pos = 0;
        let mut remain = buf.len();
        let mut temp_buf = vec![0u8; BLOCK_SIZE];

        // Read the first block
        if offset % BLOCK_SIZE as u64 != 0 || remain < BLOCK_SIZE {
            let start = cur_offset as usize % 512;
            let end = BLOCK_SIZE.min(start + remain);
            let copy_len = end - start;
            let ret = dev.read_block(cur_offset / 512, &mut temp_buf);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            buf[pos..pos + copy_len].copy_from_slice(&temp_buf[start..end]);
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                copy_len, cur_offset, pos, remain
            );
            cur_offset += copy_len as u64;
            remain -= copy_len;
            pos += copy_len;
        }

        // Read the whole block
        while remain >= BLOCK_SIZE {
            debug!(
                "cur_offset: {:?}, cur_offset % 512 = {:?} = 0!!!",
                cur_offset,
                cur_offset % 512
            );
            let ret = dev.read_block(cur_offset / 512, &mut buf[pos..pos + BLOCK_SIZE]);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                BLOCK_SIZE, cur_offset, pos, remain
            );
            cur_offset += BLOCK_SIZE as u64;
            remain -= BLOCK_SIZE;
            pos += BLOCK_SIZE;
        }

        // Read the last block
        if remain > 0 {
            debug!(
                "cur_offset: {:?}, cur_offset % 512 = {:?} = 0!!!",
                cur_offset,
                cur_offset % 512
            );
            let start = cur_offset as usize % 512;
            let copy_len = remain.min(BLOCK_SIZE as usize);
            let end = start + copy_len;
            let ret = dev.read_block(cur_offset / 512, &mut temp_buf);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            buf[pos..pos + copy_len].copy_from_slice(&temp_buf[start..end]);
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                copy_len, cur_offset, pos, remain
            );
            cur_offset += copy_len as u64;
            remain -= copy_len;
            pos += copy_len;
        }

        debug!("cur_offset - offset - buf.len() = {:?} = 0!!, remain: {:?} = 0!!, pos - buf.len() =  {:?} = 0!!", cur_offset - offset - buf.len() as u64, remain, pos - buf.len());

        Ok(buf.len())
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        debug!("Write VDA node offset: {:?}, buf: {:?}", offset, buf.len());

        let mut dev = self.transport.write();
        let mut cur_offset = offset;
        let mut pos = 0;
        let mut remain = buf.len();
        let mut temp_buf = vec![0u8; BLOCK_SIZE];

        // Write the first block
        if offset % BLOCK_SIZE as u64 != 0 || remain < BLOCK_SIZE {
            let start = cur_offset as usize % 512;
            let end = BLOCK_SIZE.min(start + remain);
            let copy_len = end - start;
            temp_buf[start..end].copy_from_slice(&buf[pos..pos + copy_len]);
            let ret = dev.write_block(cur_offset / 512, &temp_buf);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                copy_len, cur_offset, pos, remain
            );
            cur_offset += copy_len as u64;
            remain -= copy_len;
            pos += copy_len;
        }

        // Write the whole block
        while remain >= BLOCK_SIZE {
            debug!(
                "cur_offset: {:?}, cur_offset % 512 = {:?} = 0!!!",
                cur_offset,
                cur_offset % 512
            );
            let ret = dev.write_block(cur_offset / 512, &buf[pos..pos + BLOCK_SIZE]);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                BLOCK_SIZE, cur_offset, pos, remain
            );
            cur_offset += BLOCK_SIZE as u64;
            remain -= BLOCK_SIZE;
            pos += BLOCK_SIZE;
        }

        // Write the last block
        if remain > 0 {
            debug!(
                "cur_offset: {:?}, cur_offset % 512 = {:?} = 0!!!",
                cur_offset,
                cur_offset % 512
            );
            let start = cur_offset as usize % 512;
            let copy_len = remain.min(BLOCK_SIZE as usize);
            let end = start + copy_len;
            temp_buf[start..end].copy_from_slice(&buf[pos..pos + copy_len]);
            let ret = dev.write_block(cur_offset / 512, &temp_buf);
            if ret.is_err() {
                return Err(VfsError::PermissionDenied);
            }
            debug!(
                "copy_len: {:?}, cur_offset: {:?}, pos: {:?}, remain: {:?}",
                copy_len, cur_offset, pos, remain
            );
            cur_offset += copy_len as u64;
            remain -= copy_len;
            pos += copy_len;
        }

        debug!("cur_offset - offset - buf.len() = {:?} = 0!!, remain: {:?} = 0!!, pos - buf.len() =  {:?} = 0!!", cur_offset - offset - buf.len() as u64, remain, pos - buf.len());

        Ok(buf.len())
    }
}
