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
use axerrno::{ax_err, AxError, AxResult};
use axfs_vfs::{
    path::{AbsPath, RelPath},
    VfsError, VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps, VfsResult,
};
use axsync::Mutex;
use capability::Cap;
use lazy_init::LazyInit;

use crate::{
    api::FileType,
    fops::{perm_to_cap, Directory, File, OpenOptions},
};

static CURRENT_DIR_PATH: Mutex<AbsPath> = Mutex::new(AbsPath::new("/"));
static CURRENT_DIR: LazyInit<Mutex<VfsNodeRef>> = LazyInit::new();

/// mount point information
pub struct MountPoint {
    /// mount point path
    pub path: AbsPath<'static>,
    /// mounted filesystem
    pub fs: Arc<dyn VfsOps>,
}

/// fs root directory
pub struct RootDirectory {
    main_fs: Arc<dyn VfsOps>,
    mounts: Vec<MountPoint>,
}

// static ROOT_DIR: LazyInit<Arc<RootDirectory>> = LazyInit::new();

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
    /// Creates a new `RootDirectory` with the specified main filesystem.
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

/// Look up a file given an absolute path.
pub(crate) fn lookup(path: &AbsPath) -> AxResult<VfsNodeRef> {
    ROOT_DIR.clone().lookup(&path.to_rel())
}

/// Open a file given an absolute path.
pub fn open_file(path: &AbsPath, opts: &OpenOptions) -> AxResult<File> {
    debug!("open file: {} {:?}", path, opts);
    if !opts.is_valid() {
        return ax_err!(InvalidInput);
    }
    let node = match lookup(path) {
        Ok(node) => {
            if opts.create_new {
                return ax_err!(AlreadyExists);
            }
            node
        }
        Err(VfsError::NotFound) => {
            if !opts.create || !opts.create_new {
                return ax_err!(NotFound);
            }
            create_file(path)?;
            lookup(path)?
        }
        Err(e) => return Err(e),
    };

    let attr = node.get_attr()?;
    if attr.is_dir() {
        return ax_err!(IsADirectory);
    }
    let access_cap = opts.into();
    if !perm_to_cap(attr.perm()).contains(access_cap) {
        return ax_err!(PermissionDenied);
    }

    node.open()?;
    if opts.truncate {
        node.truncate(0)?;
    }
    Ok(File::new(node, access_cap, opts.append))
}

/// Open a directory given an absolute path.
pub fn open_dir(path: &AbsPath, opts: &OpenOptions) -> AxResult<Directory> {
    debug!("open dir: {}", path);
    if !opts.read {
        return ax_err!(InvalidInput);
    }
    if opts.create || opts.create_new || opts.write || opts.append || opts.truncate {
        return ax_err!(InvalidInput);
    }
    let node = lookup(path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        return ax_err!(NotADirectory);
    }
    let access_cap = opts.into();
    if !perm_to_cap(attr.perm()).contains(access_cap) {
        return ax_err!(PermissionDenied);
    }
    node.open()?;
    Ok(Directory::new(node, access_cap | Cap::EXECUTE))
}

/// Get the file attributes given an absolute path.
pub fn get_attr(path: &AbsPath) -> AxResult<VfsNodeAttr> {
    let node = lookup(path)?;
    node.get_attr()
}

/// Create a file given an absolute path.
pub(crate) fn create_file(path: &AbsPath) -> AxResult<VfsNodeRef> {
    match lookup(path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => {
            ROOT_DIR.create(&path.to_rel(), VfsNodeType::File)?;
            lookup(path)
        }
        Err(e) => Err(e),
    }
}

/// Create a directory given an absolute path.
pub(crate) fn create_dir(path: &AbsPath) -> AxResult {
    match lookup(path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => ROOT_DIR.create(&path.to_rel(), VfsNodeType::Dir),
        Err(e) => Err(e),
    }
}

/// Create a directory recursively given an absolute path.
pub(crate) fn create_dir_all(path: &AbsPath) -> AxResult {
    match lookup(path) {
        Ok(_) => ax_err!(AlreadyExists),
        Err(AxError::NotFound) => ROOT_DIR.create_recursive(&path.to_rel(), VfsNodeType::Dir),
        Err(e) => Err(e),
    }
}

/// Rename a file given an absolute path.
pub(crate) fn remove_file(path: &AbsPath) -> AxResult {
    let node = lookup(path)?;
    let attr = node.get_attr()?;
    if attr.is_dir() {
        ax_err!(IsADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        ROOT_DIR.remove(&path.to_rel())
    }
}

/// Remove a directory given an absolute path.
pub(crate) fn remove_dir(path: &AbsPath) -> AxResult {
    if ROOT_DIR.contains(path) {
        return ax_err!(PermissionDenied);
    }
    let node = lookup(path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_writable() {
        ax_err!(PermissionDenied)
    } else {
        ROOT_DIR.remove(&path.to_rel())
    }
}

/// Get current working directory.
pub(crate) fn current_dir<'a>() -> AxResult<AbsPath<'a>> {
    Ok(CURRENT_DIR_PATH.lock().clone())
}

/// Set current working directory.
pub(crate) fn set_current_dir(path: AbsPath<'static>) -> AxResult {
    let node = lookup(&path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        *CURRENT_DIR.lock() = node;
        *CURRENT_DIR_PATH.lock() = path;
        Ok(())
    }
}

/// Rename a file given an old and a new absolute path.
pub(crate) fn rename(old: &AbsPath, new: &AbsPath) -> AxResult {
    if lookup(new).is_ok() {
        ax_err!(AlreadyExists)
    } else {
        ROOT_DIR.rename(&old.to_rel(), &new.to_rel())
    }
}

#[crate_interface::def_interface]
/// Current working directory operations.
pub trait CurrentWorkingDirectoryOps {
    /// Initializes the root filesystem with the specified mount points.
    fn init_rootfs(mount_points: Vec<MountPoint>);
    /// Returns the parent node of the specified path.
    fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef;
    /// Returns the absolute path of the specified path.
    fn absolute_path(path: &str) -> AxResult<String>;
    /// Returns the current working directory.
    fn current_dir() -> AxResult<String>;
    /// Sets the current working directory.
    fn set_current_dir(path: &str) -> AxResult;
    /// get the root directory of the filesystem
    fn root_dir() -> Arc<RootDirectory>;
}

pub(crate) fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::parent_node_of, dir, path)
}

pub(crate) fn absolute_path(path: &str) -> AxResult<String> {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::absolute_path, path)
}

pub(crate) fn current_dir() -> AxResult<String> {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::current_dir)
}

pub(crate) fn set_current_dir(path: &str) -> AxResult {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::set_current_dir, path)
}

pub(crate) fn init_rootfs(mount_points: Vec<MountPoint>) {
    crate_interface::call_interface!(CurrentWorkingDirectoryOps::init_rootfs, mount_points)
}
