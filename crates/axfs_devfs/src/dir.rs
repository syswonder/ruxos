/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::collections::BTreeMap;
use alloc::sync::{Arc, Weak};
use axfs_vfs::{RelPath, VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType};
use axfs_vfs::{VfsError, VfsResult};
use spin::RwLock;

use crate::InoAllocator;

/// The directory node in the device filesystem.
///
/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct DirNode {
    ino: u64,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    children: RwLock<BTreeMap<&'static str, VfsNodeRef>>,
    ialloc: Weak<InoAllocator>,
}

impl DirNode {
    pub(super) fn new(
        ino: u64,
        parent: Option<&VfsNodeRef>,
        ialloc: Weak<InoAllocator>,
    ) -> Arc<Self> {
        let parent = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
        Arc::new(Self {
            ino,
            parent: RwLock::new(parent),
            children: RwLock::new(BTreeMap::new()),
            ialloc,
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Create a subdirectory at this directory.
    pub fn mkdir(self: &Arc<Self>, name: &'static str) -> Arc<Self> {
        let parent = self.clone() as VfsNodeRef;
        let node = Self::new(
            self.ialloc.upgrade().unwrap().alloc(),
            Some(&parent),
            self.ialloc.clone(),
        );
        self.children.write().insert(name, node.clone());
        node
    }

    /// Add a node to this directory.
    pub fn add(&self, name: &'static str, node: VfsNodeRef) {
        self.children.write().insert(name, node);
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new_dir(self.ino, 4096, 0))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.parent.read().upgrade()
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                ".." => self.parent().ok_or(VfsError::NotFound)?.lookup(&rest),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .cloned()
                    .ok_or(VfsError::NotFound)?
                    .lookup(&rest),
            }
        } else if name.is_empty() {
            Ok(self.clone() as VfsNodeRef)
        } else if name == ".." {
            self.parent().ok_or(VfsError::NotFound)
        } else {
            self.children
                .read()
                .get(name)
                .cloned()
                .ok_or(VfsError::NotFound)
        }
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let children = self.children.read();
        let mut children = children.iter().skip(start_idx.max(2) - 2);
        for (i, ent) in dirents.iter_mut().enumerate() {
            match i + start_idx {
                0 => *ent = VfsDirEntry::new(".", VfsNodeType::Dir),
                1 => *ent = VfsDirEntry::new("..", VfsNodeType::Dir),
                _ => {
                    if let Some((name, node)) = children.next() {
                        *ent = VfsDirEntry::new(name, node.get_attr().unwrap().file_type());
                    } else {
                        return Ok(i);
                    }
                }
            }
        }
        Ok(dirents.len())
    }

    fn create(&self, path: &RelPath, ty: VfsNodeType) -> VfsResult {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                ".." => self.parent().ok_or(VfsError::NotFound)?.create(&rest, ty),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .ok_or(VfsError::NotFound)?
                    .create(&rest, ty),
            }
        } else if name.is_empty() || name == ".." {
            Ok(()) // already exists
        } else {
            Err(VfsError::PermissionDenied) // do not support to create nodes dynamically
        }
    }

    fn unlink(&self, path: &RelPath) -> VfsResult {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                ".." => self.parent().ok_or(VfsError::NotFound)?.unlink(&rest),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .ok_or(VfsError::NotFound)?
                    .unlink(&rest),
            }
        } else {
            Err(VfsError::PermissionDenied) // do not support to unlink nodes dynamically
        }
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

fn split_path<'a>(path: &'a RelPath) -> (&'a str, Option<RelPath<'a>>) {
    path.find('/').map_or((path, None), |n| {
        (&path[..n], Some(RelPath::new(&path[n + 1..])))
    })
}
