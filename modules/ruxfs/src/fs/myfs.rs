/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::sync::Arc;
use axfs_vfs::VfsOps;

#[cfg(feature = "blkfs")]
use crate::dev::Disk;

/// The interface to define custom filesystems in user apps (using block device).
#[cfg(feature = "blkfs")]
#[crate_interface::def_interface]
pub trait MyFileSystemIf {
    /// Creates a new instance of the filesystem with initialization.
    fn new_myfs(disk: Disk) -> Arc<dyn VfsOps>;
}

/// The interface to define custom filesystems in user apps (without block device).
#[cfg(not(feature = "blkfs"))]
#[crate_interface::def_interface]
pub trait MyFileSystemIf {
    /// Creates a new instance of the filesystem with initialization.
    fn new_myfs() -> Arc<dyn VfsOps>;
}

#[cfg(feature = "blkfs")]
pub(crate) fn new_myfs(disk: Disk) -> Arc<dyn VfsOps> {
    crate_interface::call_interface!(MyFileSystemIf::new_myfs(disk))
}

#[cfg(not(feature = "blkfs"))]
pub(crate) fn new_myfs() -> Arc<dyn VfsOps> {
    crate_interface::call_interface!(MyFileSystemIf::new_myfs())
}
