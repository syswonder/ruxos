/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Root directory of the filesystem, where filesystem operations are distributed to the
//! appropriate filesystem based on the mount points.
//!
//! `RootDirectory::lookup_mounted_fs()` performs the distribution of operations.

use alloc::{format, string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::{
    AbsPath, RelPath, VfsError, VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType,
    VfsOps, VfsResult,
};
use spinlock::SpinNoIrq;

/// mount point information
#[derive(Clone)]
pub struct MountPoint {
    /// mount point path
    pub path: String,
    /// mounted filesystem
    pub fs: Arc<dyn VfsOps>,
}

// pub(crate) static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

impl MountPoint {
    /// create new MountPoint from data
    pub fn new(path: String, fs: Arc<dyn VfsOps>) -> Self {
        Self { path, fs }
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

/// Root directory of the main filesystem
pub struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mount_points: SpinNoIrq<Vec<MountPoint>>,
}

impl RootDirectory {
    /// Creates a new `RootDirectory` with the specified main filesystem.
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mount_points: SpinNoIrq::new(Vec::new()),
        }
    }

    /// Mount the specified filesystem at the specified path.
    pub fn mount(&self, mp: MountPoint) -> AxResult {
        info!("Root dir mounting {}", mp.path);
        if mp.path == "/" {
            return ax_err!(InvalidInput, "cannot mount root filesystem");
        }
        if !mp.path.starts_with('/') {
            return ax_err!(InvalidInput, "mount path must start with '/'");
        }
        let mut already_mount = self.mount_points.lock();
        if already_mount.iter().any(|m| m.path == mp.path) {
            return ax_err!(InvalidInput, "mount point already exists");
        }
        let rel_path = RelPath::new(&mp.path[1..]);
        // create the mount point in the main filesystem if it does not exist
        match self.main_fs.root_dir().lookup(&rel_path) {
            Ok(node) => {
                if !node.get_attr()?.is_dir() {
                    return ax_err!(InvalidInput, "mount point is not a directory");
                }
                // TODO: permission check
            }
            Err(VfsError::NotFound) => {
                self.main_fs.root_dir().create(
                    &rel_path,
                    VfsNodeType::Dir,
                    VfsNodePerm::default_dir(),
                )?;
            }
            Err(e) => {
                return Err(e);
            }
        }
        let parent = if let Some((parent_path, _)) = rel_path.rsplit_once('/') {
            self.main_fs.root_dir().lookup(&RelPath::new(parent_path))?
        } else {
            self.main_fs.root_dir()
        };
        // Ensure the parent directory exists
        mp.fs.mount(parent)?;

        already_mount.push(mp);
        Ok(())
    }

    /// Unmount the filesystem at the specified path.
    pub fn umount(&self, path: &AbsPath) {
        self.mount_points
            .lock()
            .retain(|mp| mp.path != path.to_string());
    }

    /// Check if path is a mount point
    pub fn contains(&self, path: &AbsPath) -> bool {
        self.mount_points
            .lock()
            .iter()
            .any(|mp| mp.path == path.to_string())
    }

    /// Check if path matches a mountpoint, return the index of the matched
    /// mountpoint and the matched length.
    fn lookup_mounted_fs(&self, path: &RelPath) -> (usize, usize) {
        debug!("lookup at root: {path}");
        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        for (i, mp) in self.mount_points.lock().iter().enumerate() {
            let rel_mp = RelPath::new(&mp.path[1..]);
            // path must have format: "<mountpoint>" or "<mountpoint>/..."
            if (rel_mp == *path || path.starts_with(&format!("{rel_mp}/")))
                && rel_mp.len() > max_len
            {
                max_len = mp.path.len() - 1;
                idx = i;
            }
        }

        (idx, max_len)
    }

    /// Check if path matches a mountpoint, dispatch the operation to the matched filesystem
    fn lookup_mounted_fs_then<F, T>(&self, path: &RelPath, f: F) -> AxResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &RelPath) -> AxResult<T>,
    {
        let (idx, len) = self.lookup_mounted_fs(path);
        if len > 0 {
            let mounts = self.mount_points.lock();
            f(mounts[idx].fs.clone(), &RelPath::new_trimmed(&path[len..]))
        } else {
            f(self.main_fs.clone(), path)
        }
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()
    }

    fn set_mode(&self, _mode: VfsNodePerm) -> VfsResult {
        Ok(())
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs_then(path, |fs, rest_path| fs.root_dir().lookup(rest_path))
    }

    fn create(&self, path: &RelPath, ty: VfsNodeType, mode: VfsNodePerm) -> VfsResult {
        self.lookup_mounted_fs_then(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty, mode)
            }
        })
    }

    fn unlink(&self, path: &RelPath) -> VfsResult {
        self.lookup_mounted_fs_then(path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().unlink(rest_path)
            }
        })
    }

    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult {
        let (src_idx, src_len) = self.lookup_mounted_fs(src_path);
        let (dst_idx, dst_len) = self.lookup_mounted_fs(dst_path);
        if src_idx != dst_idx {
            return ax_err!(PermissionDenied); // cannot rename across mount points
        }
        if src_path.len() == src_len {
            return ax_err!(PermissionDenied); // cannot rename mount points
        }
        if src_len > 0 {
            let mounts = self.mount_points.lock();
            mounts[src_idx].fs.root_dir().rename(
                &RelPath::new_trimmed(&src_path[src_len..]),
                &RelPath::new_trimmed(&dst_path[dst_len..]),
            )
        } else {
            self.main_fs.root_dir().rename(src_path, dst_path)
        }
    }
}
