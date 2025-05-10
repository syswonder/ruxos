/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! File system related functions.

#![cfg(feature = "fs")]

use crate::current;
use alloc::{borrow::ToOwned, format, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::VfsNodeRef;
use ruxfdtable::{FdTable, FileLike, OpenFlags};
use ruxfs::{
    fops::{lookup, CurrentWorkingDirectoryOps},
    root::{MountPoint, RootDirectory},
    AbsPath, RelPath,
};

use axerrno::{LinuxError, LinuxResult};

#[crate_interface::def_interface]
/// The interface for initializing the file system.
pub trait InitFs {
    /// Initializes the file system.
    fn add_stdios_to_fd_table(task_inner: &mut FileSystem);
}

#[cfg(not(feature = "notest"))]
struct InitFsDefaultImpl;

#[cfg(not(feature = "notest"))]
#[crate_interface::impl_interface]
impl InitFs for InitFsDefaultImpl {
    fn add_stdios_to_fd_table(_task_inner: &mut FileSystem) {
        // do nothing
    }
}

/// Get the file object associated with the given file descriptor.
pub fn get_file_like(fd: i32) -> LinuxResult<Arc<dyn FileLike>> {
    // let _exec = *MUST_EXEC;
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    if let Some(fs) = binding_fs.as_mut() {
        fs.fd_table
            .get(fd as usize)
            .cloned()
            .ok_or(LinuxError::EBADF)
    } else {
        Err(LinuxError::EBADF)
    }
}

/// Adds a file like object to the file descriptor table and returns the file descriptor.
/// Actually there only `CLOEXEC` flag in options works.
pub fn add_file_like(f: Arc<dyn FileLike>, flags: OpenFlags) -> LinuxResult<i32> {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().expect("No fd table found").fd_table;
    let fd = fd_table.add(f, flags).ok_or(LinuxError::EMFILE)?;
    Ok(fd as _)
}

/// Removes a file like object from the file descriptor table.
pub fn close_file_like(fd: i32) -> LinuxResult {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().unwrap().fd_table;

    let file = fd_table.remove(fd as usize).ok_or(LinuxError::EBADF)?;

    // drop the binding_fs to release the lock, as some operations
    // when closing a file may need to reschedule the task.(e.g. SOCKET_CLOSE)
    drop(binding_fs);
    drop(file);

    Ok(())
}

/// Get current task umask
pub fn get_umask() -> u16 {
    current().fs.lock().as_ref().unwrap().umask.get()
}

/// Replace current task umask
pub fn replace_umask(new_mask: u16) -> u16 {
    current()
        .fs
        .lock()
        .as_mut()
        .unwrap()
        .umask
        .replace(new_mask)
}

/// A struct representing a file system object.
#[derive(Clone)]
pub struct FileSystem {
    /// The file descriptor table.
    pub fd_table: FdTable,
    /// The current working directory.
    pub current_path: AbsPath<'static>,
    /// The current directory.
    pub current_dir: VfsNodeRef,
    /// The root directory.
    pub root_dir: Arc<RootDirectory>,
    /// file mode creation maskq
    umask: UMask,
}

impl FileSystem {
    /// Closes all file objects in the file descriptor table.
    pub fn close_all_files(&mut self) {
        self.fd_table.close_all_files();
    }
}

/// Initializes the file system.
pub fn init_rootfs(mount_points: Vec<MountPoint>) {
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

    let root_dir_arc = Arc::new(root_dir);

    let mut fs = FileSystem {
        fd_table: FdTable::default(),
        current_path: AbsPath::new_owned("/".to_owned()),
        current_dir: root_dir_arc.clone(),
        root_dir: root_dir_arc.clone(),
        umask: UMask::new(),
    };

    // TODO: make a more clear interface for adding stdios to fd table when not in unit tests
    let fs_mutable = &mut fs;
    crate_interface::call_interface!(InitFs::add_stdios_to_fd_table, fs_mutable);

    current().fs.lock().replace(fs);
}

fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
    if path.starts_with('/') {
        current().fs.lock().as_mut().unwrap().root_dir.clone()
    } else {
        dir.cloned()
            .unwrap_or_else(|| current().fs.lock().as_mut().unwrap().current_dir.clone())
    }
}

/// Returns the absolute path of the given path.
pub fn absolute_path(path: &str) -> AxResult<AbsPath<'static>> {
    if path.starts_with('/') {
        Ok(AbsPath::new_canonicalized(path))
    } else {
        Ok(current()
            .fs
            .lock()
            .as_mut()
            .unwrap()
            .current_path
            .join(&RelPath::new_canonicalized(path)))
    }
}

/// Returns the current directory.
pub fn current_dir() -> AxResult<AbsPath<'static>> {
    Ok(current().fs.lock().as_mut().unwrap().current_path.clone())
}

/// Sets the current directory.
pub fn set_current_dir(path: AbsPath<'static>) -> AxResult {
    let node = lookup(&path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        current().fs.lock().as_mut().unwrap().current_dir = node;
        current().fs.lock().as_mut().unwrap().current_path = path;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
struct UMask(u16);

impl UMask {
    fn new() -> Self {
        Self(0o022)
    }

    fn get(&self) -> u16 {
        self.0
    }

    fn replace(&mut self, new_mask: u16) -> u16 {
        let new_mask = new_mask & 0o777;
        let old_mask = self.0;
        self.0 = new_mask;
        old_mask
    }
}

struct CurrentWorkingDirectoryImpl;

#[crate_interface::impl_interface]
impl CurrentWorkingDirectoryOps for CurrentWorkingDirectoryImpl {
    fn init_rootfs(mount_points: Vec<MountPoint>) {
        init_rootfs(mount_points)
    }
    fn parent_node_of(dir: Option<&VfsNodeRef>, path: &RelPath) -> VfsNodeRef {
        parent_node_of(dir, path)
    }
    fn absolute_path(path: &str) -> AxResult<AbsPath<'static>> {
        absolute_path(path)
    }
    fn current_dir() -> AxResult<AbsPath<'static>> {
        current_dir()
    }
    fn set_current_dir(path: AbsPath<'static>) -> AxResult {
        set_current_dir(path)
    }
    fn root_dir() -> Arc<RootDirectory> {
        current()
            .fs
            .lock()
            .as_mut()
            .expect("No filesystem found")
            .root_dir
            .clone()
    }
}
