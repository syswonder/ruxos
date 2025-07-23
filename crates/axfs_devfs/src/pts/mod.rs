/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! A pseudoterminal (Pty) consists of two endpoints:
//! ​Master: Controls the terminal session (handled by PtyMaster)
//! Slave: Emulates physical terminal hardware for programs
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use axfs_vfs::{VfsNodeRef, VfsOps, VfsResult};
use core::cmp::Reverse;
use master::PtyMaster;
use ptmx::Ptmx;
use root::PtsRootInode;
use slave::{PtySlave, PtySlaveInode};
use spin::{mutex::Mutex, once::Once};

mod master;
mod ptmx;
mod root;
mod slave;

/// A virtual filesystem managing pseudo-terminal (PTY) devices.
///
/// This filesystem exposes PTY master/slave pairs through device nodes
/// (typically /dev/ptmx for master and /dev/pts/* for slaves).
pub struct PtsFileSystem {
    /// Root inode of the PTY filesystem (usually mounted at /dev/pts)
    root: Arc<PtsRootInode>,
    /// Reusable index allocator for slave devices
    idx_allocator: Mutex<RecycleAllocator>,
}

/// Used for allocating index
struct RecycleAllocator {
    /// Current max id allocated
    current_max_id: usize,
    /// Hold deallocated id, will be recycled first when alloc happen
    recycled: BinaryHeap<Reverse<usize>>,
}

impl RecycleAllocator {
    /// Create an empty `RecycleAllocator`
    fn new(init_val: usize) -> Self {
        RecycleAllocator {
            current_max_id: init_val,
            recycled: BinaryHeap::new(),
        }
    }

    /// Allocate an id
    fn alloc(&mut self) -> usize {
        if let Some(Reverse(id)) = self.recycled.pop() {
            id
        } else {
            self.current_max_id += 1;
            self.current_max_id - 1
        }
    }

    /// Recycle an id
    fn dealloc(&mut self, id: usize) {
        debug_assert!(id < self.current_max_id);
        debug_assert!(
            !self.recycled.iter().any(|iid| iid.0 == id),
            "id {id} has been deallocated!",
        );
        self.recycled.push(Reverse(id));
    }
}

/// Inode number for root directory
pub(crate) const PTS_ROOT_INO: u64 = 1;
/// Inode number for ptmx character device
pub(crate) const PTS_PTMX_INO: u64 = 2;

/// Global singleton instance of the PTY filesystem
pub(crate) static PTS_FS: Once<Arc<PtsFileSystem>> = Once::new();

/// Initialize the PTY filesystem and return root inode
pub fn init_pts() -> Arc<PtsRootInode> {
    let ptsfs = PtsFileSystem::new();
    let root = ptsfs.root.clone();
    PTS_FS.call_once(|| ptsfs);
    root
}

impl PtsFileSystem {
    /// Create a new PTY filesystem instance
    pub fn new() -> Arc<Self> {
        // using cyclic reference pattern to break dependency between filesystem and root inode
        Arc::new_cyclic(|weak_fs| Self {
            root: PtsRootInode::new(weak_fs.clone()),
            idx_allocator: Mutex::new(RecycleAllocator::new(0)),
        })
    }

    /// Get the ptmx device (pseudo-terminal master multiplexer)
    pub fn ptmx(&self) -> Arc<Ptmx> {
        self.root.ptmx().clone()
    }

    /// Allocate a new PTY master/slave pair
    /// 1. Allocates new inode number
    /// 2. Creates master device handle
    /// 3. Creates associated slave device
    /// 4. Registers slave in the filesystem
    pub fn allocate_pty(&self) -> Arc<PtyMaster> {
        let idx = self.idx_allocator.lock().alloc();
        let master = Arc::new(PtyMaster::new(self.root.ptmx().clone(), idx as _));
        let slave = Arc::new(PtySlave::new(&master));
        let slave_inode = Arc::new(PtySlaveInode::new(slave));
        self.root.push_slave(idx as _, slave_inode);
        master
    }

    /// Remove a PTY slave device by index
    pub fn remove_pty(&self, idx: usize) {
        self.root.remove_slave(idx);
        self.idx_allocator.lock().dealloc(idx);
    }
}

impl VfsOps for PtsFileSystem {
    fn mount(&self, parent: VfsNodeRef) -> VfsResult {
        self.root.set_parent(&parent);
        Ok(())
    }

    fn root_dir(&self) -> VfsNodeRef {
        self.root.clone()
    }
}
