/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
//! Unix socket node in vfs
use alloc::sync::{Arc, Weak};
use axerrno::{ax_err, LinuxError, LinuxResult};
use axfs_vfs::{
    impl_vfs_non_dir_default, AbsPath, RelPath, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType,
    VfsResult,
};
use ruxfs::fops::lookup;
use spin::rwlock::RwLock;

use crate::socket::Socket;

/// Represents a filesystem node for a UNIX domain socket.
/// This connects the socket abstraction with the VFS (Virtual Filesystem) layer,
/// allowing sockets to be addressed via filesystem paths like `/tmp/sock`.
pub struct SocketNode {
    attr: RwLock<VfsNodeAttr>,
    /// Weak reference to the actual socket implementation.
    /// Using Weak avoids circular references between the filesystem and socket layers.
    bound_socket: Weak<Socket>,
}

impl SocketNode {
    /// Creates a new socket filesystem node bound to a specific socket.
    fn new(socket: Arc<Socket>) -> Self {
        Self {
            // FIXME: use a proper inode number
            attr: RwLock::new(VfsNodeAttr::new(
                520,
                VfsNodePerm::default_socket(),
                VfsNodeType::Socket,
                0,
                0,
            )),
            bound_socket: Arc::downgrade(&socket),
        }
    }

    /// Retrieves the bound socket as a strong reference.
    /// Panics if the socket has been dropped (should never happen in correct usage).
    pub fn bound_socket(&self) -> Arc<Socket> {
        self.bound_socket.upgrade().unwrap()
    }
}

impl VfsNodeOps for SocketNode {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(*self.attr.read())
    }

    fn set_mode(&self, mode: VfsNodePerm) -> VfsResult {
        self.attr.write().set_perm(mode);
        Ok(())
    }

    fn read_at(&self, _offset: u64, _buf: &mut [u8]) -> VfsResult<usize> {
        ax_err!(Unsupported, "Socket read_at method Unsupported")
    }

    fn write_at(&self, _offset: u64, _buf: &[u8]) -> VfsResult<usize> {
        ax_err!(Unsupported, "Socket write_at method Unsupported")
    }

    fn fsync(&self) -> VfsResult {
        ax_err!(Unsupported, "Socket fsync method Unsupported")
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        ax_err!(Unsupported, "Socket truncate method Unsupported")
    }

    impl_vfs_non_dir_default! {}
}

/// Binds a UNIX domain socket to a filesystem path.
pub fn bind_socket_node(socket: Arc<Socket>, abs_path: &AbsPath) -> LinuxResult {
    if let Some((dir_path, name)) = abs_path.rsplit_once('/') {
        let dir_node = lookup(&AbsPath::new(dir_path))?;
        let socket_node = Arc::new(SocketNode::new(socket));
        dir_node.create_socket_node(&RelPath::new(name), socket_node)?;
        Ok(())
    } else {
        // A component in the directory prefix of the socket pathname does not exist.
        Err(LinuxError::ENOENT)
    }
}
