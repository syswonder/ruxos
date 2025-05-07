/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! RAM filesystem used by [ArceOS](https://github.com/rcore-os/arceos).
//!
//! The implementation is based on [`axfs_vfs`].

#![cfg_attr(not(test), no_std)]

extern crate alloc;

mod dir;
mod file;

#[cfg(test)]
mod tests;

pub use self::dir::DirNode;
pub use self::file::FileNode;

use alloc::sync::Arc;
use axfs_vfs::{AbsPath, VfsNodePerm, VfsNodeRef, VfsOps, VfsResult};
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

/// A RAM filesystem that implements [`axfs_vfs::VfsOps`].
pub struct RamFileSystem {
    parent: Once<VfsNodeRef>,
    root: Arc<DirNode>,
    _ialloc: Arc<InoAllocator>,
}

impl RamFileSystem {
    /// Create a new instance.
    pub fn new() -> Self {
        let ialloc = Arc::new(InoAllocator::new(0));
        Self {
            parent: Once::new(),
            root: DirNode::new(
                ialloc.alloc(),
                VfsNodePerm::default_dir(),
                None,
                Arc::downgrade(&ialloc),
            ),
            _ialloc: ialloc,
        }
    }

    /// Returns the root directory node in [`Arc<DirNode>`](DirNode).
    pub fn root_dir_node(&self) -> Arc<DirNode> {
        self.root.clone()
    }
}

impl VfsOps for RamFileSystem {
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

impl Default for RamFileSystem {
    fn default() -> Self {
        Self::new()
    }
}
