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
use alloc::{string::String, vec::Vec};

use axfs_vfs::{
    RelPath, VfsDirEntry, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType,
};
use axfs_vfs::{VfsError, VfsResult};
use ruxfifo::FifoNode;
use spin::rwlock::RwLock;

use crate::file::FileNode;
use crate::InoAllocator;

/// The directory node in the RAM filesystem.
///
/// It implements [`axfs_vfs::VfsNodeOps`].
pub struct DirNode {
    attr: RwLock<VfsNodeAttr>,
    this: Weak<DirNode>,
    parent: RwLock<Weak<dyn VfsNodeOps>>,
    children: RwLock<BTreeMap<String, VfsNodeRef>>,
    ialloc: Weak<InoAllocator>,
}

impl DirNode {
    pub(super) fn new(
        ino: u64,
        mode: VfsNodePerm,
        parent: Option<Weak<dyn VfsNodeOps>>,
        ialloc: Weak<InoAllocator>,
    ) -> Arc<Self> {
        Arc::new_cyclic(|this| Self {
            attr: RwLock::new(VfsNodeAttr::new(ino, mode, VfsNodeType::Dir, 4096, 0)),
            this: this.clone(),
            parent: RwLock::new(parent.unwrap_or_else(|| Weak::<Self>::new())),
            children: RwLock::new(BTreeMap::new()),
            ialloc,
        })
    }

    pub(super) fn set_parent(&self, parent: Option<&VfsNodeRef>) {
        *self.parent.write() = parent.map_or(Weak::<Self>::new() as _, Arc::downgrade);
    }

    /// Returns a string list of all entries in this directory.
    pub fn get_entries(&self) -> Vec<String> {
        self.children.read().keys().cloned().collect()
    }

    /// Checks whether a node with the given name exists in this directory.
    pub fn exist(&self, name: &str) -> bool {
        self.children.read().contains_key(name)
    }

    /// Creates a new node with the given name and type in this directory.
    pub fn create_node(&self, name: &str, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        if self.exist(name) {
            log::error!("AlreadyExists {name}");
            return Err(VfsError::AlreadyExists);
        }
        let ino = self.ialloc.upgrade().unwrap().alloc();
        let node: VfsNodeRef = match ty {
            VfsNodeType::File => Arc::new(FileNode::new(ino, mode)),
            VfsNodeType::Fifo => Arc::new(FifoNode::new(ino, mode)),
            VfsNodeType::Dir => Self::new(ino, mode, Some(self.this.clone()), self.ialloc.clone()),
            _ => return Err(VfsError::Unsupported),
        };
        self.children.write().insert(name.into(), node);
        Ok(())
    }

    /// Removes a node by the given name in this directory.
    pub fn remove_node(&self, name: &str) -> VfsResult {
        let mut children = self.children.write();
        let node = children.get(name).ok_or(VfsError::NotFound)?;
        if let Some(dir) = node.as_any().downcast_ref::<DirNode>() {
            if !dir.children.read().is_empty() {
                return Err(VfsError::DirectoryNotEmpty);
            }
        }
        children.remove(name);
        Ok(())
    }
}

impl VfsNodeOps for DirNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(*self.attr.read())
    }
    fn set_mode(&self, mode: VfsNodePerm) -> VfsResult {
        self.attr.write().set_perm(mode);
        Ok(())
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

    fn create(&self, path: &RelPath, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        let (name, rest) = split_path(path);
        if let Some(rest) = rest {
            match name {
                ".." => self
                    .parent()
                    .ok_or(VfsError::NotFound)?
                    .create(&rest, ty, mode),
                _ => self
                    .children
                    .read()
                    .get(name)
                    .ok_or(VfsError::NotFound)?
                    .create(&rest, ty, mode),
            }
        } else if name.is_empty() || name == ".." {
            Ok(()) // already exists
        } else {
            self.create_node(name, ty, mode)
        }
    }

    fn create_socket_node(&self, name: &RelPath, node: VfsNodeRef) -> VfsResult {
        if self.exist(name) {
            log::error!("AlreadyExists {name}");
            return Err(VfsError::AlreadyExists);
        }
        self.children.write().insert(name.as_str().into(), node);
        Ok(())
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
        } else if name.is_empty() || name == ".." {
            Err(VfsError::InvalidInput) // remove '.' or '..
        } else {
            self.remove_node(name)
        }
    }

    axfs_vfs::impl_vfs_dir_default! {}
}

fn split_path<'a>(path: &'a RelPath) -> (&'a str, Option<RelPath<'a>>) {
    path.find('/').map_or((path, None), |n| {
        (&path[..n], Some(RelPath::new(&path[n + 1..])))
    })
}
