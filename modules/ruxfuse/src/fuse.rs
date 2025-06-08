/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

// #![cfg(feature = "multitask")]

use alloc::sync::{Arc, Weak};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ruxtask::{current, WaitQueue};
use spinlock::SpinNoIrq;
use core::sync::atomic::{AtomicI32, AtomicU64, Ordering};
use log::*;

use axfs_vfs::{VfsDirEntry, VfsError, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use spin::{once::Once, RwLock};
use ruxfs::fuse_st::{
    FuseAccessIn, FuseAttr, FuseAttrOut, FuseCreateIn, FuseDirent, FuseEntryOut, FuseFlushIn,
    FuseForgetIn, FuseGetattrIn, FuseInHeader, FuseInitIn, FuseInitOut, FuseLseekIn, FuseLseekOut,
    FuseMkdirIn, FuseMknodIn, FuseOpcode, FuseOpenIn, FuseOpenOut, FuseOutHeader, FuseReadIn,
    FuseReleaseIn, FuseRename2In, FuseRenameIn, FuseStatfsOut, FuseWriteIn, FuseWriteOut
};
use ruxfs::devfuse::{FUSEFLAG, FUSE_VEC};

// pub static mut UNIQUE_ID: u64 = 0;
pub static UNIQUE_ID: AtomicU64 = AtomicU64::new(0);
pub static NEWID: AtomicI32 = AtomicI32::new(-1);
pub static INITFLAG: AtomicI32 = AtomicI32::new(1);
pub static WQ: WaitQueue = WaitQueue::new();

/// It implements [`axfs_vfs::VfsOps`].
pub struct FuseFS {
    parent: Once<VfsNodeRef>,
    root: Arc<FuseNode>,
}

impl FuseFS {
    /// Create a new instance.
    pub fn new() -> Self {
        info!("fusefs new...");
        // let parent: Weak<dyn VfsNodeOps> = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
        Self {
            parent: Once::new(),
            root: FuseNode::new(None, 1, FuseAttr::default(), 0, 0),
        }
    }
}

/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct FuseNode {
    this: Weak<FuseNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    inode: SpinNoIrq<u64>,
    attr: SpinNoIrq<FuseAttr>,
    nlink: SpinNoIrq<u32>,
    flags: SpinNoIrq<u32>,
    fh: SpinNoIrq<u64>,
}

impl FuseNode {
    /// Create a new instance.
    pub(super) fn new(parent: Option<Weak<dyn VfsNodeOps>>, inode: u64, attr: FuseAttr, nlink: u32, fh: u64) -> Arc<Self> {
        info!("fuse_node new inode: {:?}, nlink: {:?}", inode, nlink);
        Arc::new_cyclic(|this| Self {
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            inode: SpinNoIrq::new(inode),
            attr: SpinNoIrq::new(attr),
            nlink: SpinNoIrq::new(nlink),
            flags: SpinNoIrq::new(0x8000),
            fh: SpinNoIrq::new(fh),
        })
    }
    
}

impl VfsNodeOps for FuseNode {
    
    
}

fn split_path(path: &str) -> (&str, Option<&str>) {
    let trimmed_path = path.trim_start_matches('/');
    trimmed_path.find('/').map_or((trimmed_path, None), |n| {
        (&trimmed_path[..n], Some(&trimmed_path[n + 1..]))
    })
}

pub fn fusefs() -> Arc<FuseFS> {
    debug!("fusefs newfs here...");
    Arc::new(FuseFS::new())
}