/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Root directory of the filesystem
//!
//! TODO: it doesn't work very well if the mount points have containment relationships.

use alloc::{format, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::{
    path::{AbsPath, RelPath},
    VfsError, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult,
};
use axsync::Mutex;
use lazy_init::LazyInit;

use crate::api::FileType;

pub(crate) static CURRENT_DIR_PATH: Mutex<AbsPath> = Mutex::new(AbsPath::new("/"));
pub(crate) static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

/// mount point information
pub struct MountPoint {
    path: AbsPath<'static>,
    fs: Arc<dyn VfsOps>,
}

pub(crate) struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

pub(crate) static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

impl MountPoint {
    /// create new MountPoint from data
    pub fn new(path: AbsPath<'static>, fs: Arc<dyn VfsOps>) -> Self {
        Self { path, fs }
    }
}

impl Drop for MountPoint {
    fn drop(&mut self) {
        self.fs.umount().ok();
    }
}

impl RootDirectory {
    pub const fn new(main_fs: Arc<dyn VfsOps>) -> Self {
        Self {
            main_fs,
            mounts: Vec::new(),
        }
    }

    pub fn mount(&mut self, path: AbsPath<'static>, fs: Arc<dyn VfsOps>) -> AxResult {
        if path == AbsPath::new("/") {
            return ax_err!(InvalidInput, "cannot mount root filesystem");
        }
        if self.mounts.iter().any(|mp| mp.path == path) {
            return ax_err!(InvalidInput, "mount point already exists");
        }
        // create the mount point in the main filesystem if it does not exist
        match self.main_fs.root_dir().lookup(&path.to_rel()) {
            Ok(_) => {}
            Err(err_code) => {
                if err_code == VfsError::NotFound {
                    self.main_fs
                        .root_dir()
                        .create(&path.to_rel(), FileType::Dir)?;
                }
            }
        }
        fs.mount(&path, self.main_fs.root_dir().lookup(&path.to_rel())?)?;
        self.mounts.push(MountPoint::new(path, fs));
        Ok(())
    }

    pub fn _umount(&mut self, path: &AbsPath) {
        self.mounts.retain(|mp| mp.path != *path);
    }

    pub fn contains(&self, path: &AbsPath) -> bool {
        self.mounts.iter().any(|mp| mp.path == *path)
    }

    fn lookup_mounted_fs<F, T>(&self, path: &RelPath, f: F) -> AxResult<T>
    where
        F: FnOnce(Arc<dyn VfsOps>, &RelPath) -> AxResult<T>,
    {
        debug!("lookup at root: {}", path);
        let mut idx = 0;
        let mut max_len = 0;

        // Find the filesystem that has the longest mounted path match
        // TODO: more efficient, e.g. trie
        for (i, mp) in self.mounts.iter().enumerate() {
            // skip the first '/'
            if path.starts_with(&mp.path[1..]) && mp.path.len() - 1 > max_len {
                max_len = mp.path.len() - 1;
                idx = i;
            }
        }

        if max_len == 0 {
            f(self.main_fs.clone(), path) // not matched any mount point
        } else {
            f(self.mounts[idx].fs.clone(), &RelPath::new(&path[max_len..])) // matched at `idx`
        }
    }
}

impl VfsNodeOps for RootDirectory {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.main_fs.root_dir().get_attr()
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        self.lookup_mounted_fs(path, |fs, rest_path| fs.root_dir().lookup(rest_path))
    }

    fn create(&self, path: &RelPath, ty: VfsNodeType) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                Ok(()) // already exists
            } else {
                fs.root_dir().create(rest_path, ty)
            }
        })
    }

    fn remove(&self, path: &RelPath) -> VfsResult {
        self.lookup_mounted_fs(path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot remove mount points
            } else {
                fs.root_dir().remove(rest_path)
            }
        })
    }

    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult {
        self.lookup_mounted_fs(src_path, |fs, rest_path| {
            if rest_path.is_empty() {
                ax_err!(PermissionDenied) // cannot rename mount points
            } else {
                fs.root_dir().rename(rest_path, dst_path)
            }
        })
    }
}

pub(crate) fn init_rootfs(mount_points: Vec<MountPoint>) {
    let main_fs = mount_points
        .first()
        .expect("No filesystem found")
        .fs
        .clone();
    let mut root_dir = RootDirectory::new(main_fs);

    for mp in mount_points.iter().skip(1) {
        let vfsops = mp.fs.clone();
        let message = format!("failed to mount filesystem at {}", mp.path);
        info!("mounting {}", mp.path);
        root_dir.mount(mp.path.clone(), vfsops).expect(&message);
    }

    ROOT_DIR.init_by(Arc::new(root_dir));
    CURRENT_DIR.init_by(Mutex::new(ROOT_DIR.clone()));
    *CURRENT_DIR_PATH.lock() = AbsPath::new("/");
}
