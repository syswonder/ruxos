/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Device filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is based on [`axfs_vfs`].

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod dir;
mod null;
mod random;
mod zero;

#[cfg(test)]
mod tests;

pub use self::dir::DirNode;
pub use self::null::NullDev;
pub use self::random::RandomDev;
pub use self::zero::ZeroDev;

use alloc::sync::Arc;
use axfs_vfs::{AbsPath, VfsNodeRef, VfsOps, VfsResult};
use core::sync::atomic::AtomicU64;
use spin::once::Once;

/// An auto-increasing inode number allocator.
pub struct InoAllocator {
    current: AtomicU64,
}

impl InoAllocator {
    /// Create a new allocator instance.
    pub fn new(start: u64) -> Self {
        Self {
            current: AtomicU64::new(start),
        }
    }

    /// Allocate a new inode number.
    pub fn alloc(&self) -> u64 {
        self.current
            .fetch_add(1, core::sync::atomic::Ordering::SeqCst)
    }
}

/// A device filesystem that implements [`axfs_vfs::VfsOps`].
pub struct DeviceFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
    _ialloc: Arc<InoAllocator>,
}

impl DeviceFileSystem {
    /// Create a new instance.
    pub fn new() -> Self {
        let ialloc = Arc::new(InoAllocator::new(10));
        Self {
            parent: Once::new(),
            root: DirNode::new(2, None, Arc::downgrade(&ialloc)),
            _ialloc: ialloc,
        }
    }

    /// Create a subdirectory at the root directory.
    pub fn mkdir(&self, name: &'static str) -> Arc<DirNode> {
        self.root.mkdir(name)
    }

    /// Add a node to the root directory.
    ///
    /// The node must implement [`axfs_vfs::VfsNodeOps`], and be wrapped in [`Arc`].
    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.root.add(name, node);
    }
}

impl VfsOps for DeviceFileSystem {
    fn mount(&self, _path: &AbsPath, mount_point: VfsNodeRef) -> VfsResult {
        if let Some(parent) = mount_point.parent() {
            self.root.set_parent(Some(self.parent.call_once(|| parent)));
        } else {
            self.root.set_parent(None);
        }
        Ok(())
    }

    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}

impl Default for DeviceFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
