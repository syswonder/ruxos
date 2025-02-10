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
use alloc::{format, string::String, sync::Arc, vec::Vec};
use axerrno::{ax_err, AxResult};
use axfs_vfs::VfsNodeRef;
use bitmaps::Bitmap;
use flatten_objects::FlattenObjects;
use ruxfdtable::FileLike;
use ruxfs::{
    fops,
    root::{lookup, CurrentWorkingDirectoryOps, RootDirectory},
    MountPoint,
};

use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use ruxfdtable::RuxStat;
use spin::RwLock;

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

/// Initializes the file system.
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
pub fn add_file_like(f: Arc<dyn FileLike>, options: fops::OpenOptions) -> LinuxResult<i32> {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().expect("No fd table found").fd_table;
    let fd = fd_table.add(f, options).ok_or(LinuxError::EMFILE)?;
    Ok(fd as _)
}

/// Removes a file like object from the file descriptor table.
pub fn close_file_like(fd: i32) -> LinuxResult {
    let binding_task = current();
    let mut binding_fs = binding_task.fs.lock();
    let fd_table = &mut binding_fs.as_mut().unwrap().fd_table;
    fd_table.remove(fd as usize)?;
    Ok(())
}

/// A struct representing a file object.
pub struct File {
    /// The inner file object.
    pub inner: RwLock<ruxfs::fops::File>,
}

impl File {
    /// Creates a new file object with the given inner file object.
    pub fn new(inner: ruxfs::fops::File) -> Self {
        Self {
            inner: RwLock::new(inner),
        }
    }

    /// Adds the file object to the file descriptor table and returns the file descriptor.
    pub fn add_to_fd_table(self, options: fops::OpenOptions) -> LinuxResult<i32> {
        add_file_like(Arc::new(self), options)
    }

    /// Creates a new file object from the given file descriptor.
    pub fn from_fd(fd: i32) -> LinuxResult<Arc<Self>> {
        let f = get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.write().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.write().write(buf)?)
    }

    fn flush(&self) -> LinuxResult {
        Ok(self.inner.write().flush()?)
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.read().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;

        // Inode of files, for musl dynamic linker.
        // WARN: there will be collision for files with the same size.
        // TODO: implement real inode.
        let st_ino = metadata.size() + st_mode as u64;

        let res = RuxStat {
            st_ino,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        };

        Ok(res)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// A struct representing a directory object.
pub struct Directory {
    /// The inner directory object.
    pub inner: RwLock<ruxfs::fops::Directory>,
}

impl Directory {
    /// Creates a new directory object with the given inner directory object.
    pub fn new(inner: ruxfs::fops::Directory) -> Self {
        Self {
            inner: RwLock::new(inner),
        }
    }

    /// Adds the directory object to the file descriptor table and returns the file descriptor.
    pub fn add_to_fd_table(self, flags: fops::OpenOptions) -> LinuxResult<i32> {
        add_file_like(Arc::new(self), flags)
    }

    /// Creates a new directory object from the given file descriptor.
    pub fn from_fd(fd: i32) -> LinuxResult<Arc<Self>> {
        let f = get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for Directory {
    fn read(&self, _buf: &mut [u8]) -> LinuxResult<usize> {
        Err(LinuxError::EACCES)
    }

    fn write(&self, _buf: &[u8]) -> LinuxResult<usize> {
        Err(LinuxError::EACCES)
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.read().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(RuxStat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
            pollhup: false,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// Maximum number of files per process
pub const RUX_FILE_LIMIT: usize = 1024;

/// A struct representing a file system object.
#[derive(Clone)]
pub struct FileSystem {
    /// The file descriptor table.
    pub fd_table: FdTable,
    /// The current working directory.
    pub current_path: String,
    /// The current directory.
    pub current_dir: VfsNodeRef,
    /// The root directory.
    pub root_dir: Arc<RootDirectory>,
}

/// A table of file descriptors, containing a collection of file objects and their associated flags(CLOEXEC).
pub struct FdTable {
    /// A collection of file objects, indexed by their file descriptor numbers.
    files: FlattenObjects<Arc<dyn FileLike>, RUX_FILE_LIMIT>,
    /// A bitmap for tracking `FD_CLOEXEC` flags for each file descriptor.
    /// If a bit is set, the corresponding file descriptor has the `FD_CLOEXEC` flag enabled.
    cloexec_bitmap: Bitmap<RUX_FILE_LIMIT>,
}

impl Clone for FdTable {
    fn clone(&self) -> Self {
        // get all file descriptors from the original file system to copy them to the new one
        // TODO: make this more efficient by only copying the used file descriptors
        let mut new_files = FlattenObjects::new();
        for fd in 0..self.files.capacity() {
            if let Some(f) = self.files.get(fd) {
                new_files.add_at(fd, f.clone()).unwrap();
            }
        }
        Self {
            files: new_files,
            cloexec_bitmap: self.cloexec_bitmap,
        }
    }
}

impl Default for FdTable {
    fn default() -> Self {
        FdTable {
            files: FlattenObjects::new(),
            cloexec_bitmap: Bitmap::new(),
        }
    }
}

impl FdTable {
    /// Retrieves the file object associated with the given file descriptor (fd).
    ///
    /// Returns `Some` with the file object if the file descriptor exists, or `None` if not.
    pub fn get(&self, fd: usize) -> Option<&Arc<dyn FileLike>> {
        self.files.get(fd)
    }

    /// Adds a new file object to the table and associates it with a file descriptor.
    ///
    /// Also sets the `FD_CLOEXEC` flag for the file descriptor based on the `flags` argument.
    /// Returns the assigned file descriptor number (`fd`) if successful, or `None` if the table is full.
    pub fn add(&mut self, file: Arc<dyn FileLike>, flags: fops::OpenOptions) -> Option<usize> {
        if let Some(fd) = self.files.add(file) {
            debug_assert!(!self.cloexec_bitmap.get(fd));
            if flags.is_cloexec() {
                self.cloexec_bitmap.set(fd, true);
            }
            Some(fd)
        } else {
            None
        }
    }

    /// Adds a file object to the table at a specific file descriptor.
    /// It won't be add if the specified fd in the fdtable already exists
    pub fn add_at(&mut self, fd: usize, file: Arc<dyn FileLike>) -> Option<usize> {
        self.files.add_at(fd, file)
    }

    /// Retrieves the `FD_CLOEXEC` flag for the specified file descriptor.
    ///
    /// Returns `true` if the flag is set, otherwise `false`.
    pub fn get_cloexec(&self, fd: usize) -> bool {
        self.cloexec_bitmap.get(fd)
    }

    /// Sets the `FD_CLOEXEC` flag for the specified file descriptor.
    pub fn set_cloexec(&mut self, fd: usize, cloexec: bool) {
        self.cloexec_bitmap.set(fd, cloexec);
    }

    /// Removes a file descriptor from the table.
    ///
    /// This will clear the `FD_CLOEXEC` flag for the file descriptor and remove the file object.
    pub fn remove(&mut self, fd: usize) -> LinuxResult {
        self.cloexec_bitmap.set(fd, false);
        // use map_or because RAII. the Arc should be released here. You should not use the return Arc
        self.files
            .remove(fd)
            .map_or(Err(LinuxError::EBADF), |_| Ok(()))
    }

    /// Closes all file descriptors with the `FD_CLOEXEC` flag set.
    ///
    /// This will remove all file descriptors marked for close-on-exec from the table.
    pub fn do_close_on_exec(&mut self) {
        for fd in self.cloexec_bitmap.into_iter() {
            self.files.remove(fd);
        }
        self.cloexec_bitmap = Bitmap::new()
    }

    /// Duplicates a file descriptor and returns a new file descriptor.
    ///
    /// The two file descriptors do not share file descriptor flags (the close-on-exec flag).
    /// The close-on-exec flag (FD_CLOEXEC; see fcntl(2)) for the duplicate descriptor is off.
    pub fn dup(&mut self, fd: usize) -> LinuxResult<usize> {
        let f = self.files.get(fd).ok_or(LinuxError::EBADF)?.clone();
        let new_fd = self.files.add(f).ok_or(LinuxError::EMFILE)?;
        debug_assert!(!self.cloexec_bitmap.get(new_fd));
        Ok(new_fd)
    }

    /// Duplicates a file descriptor to a specific file descriptor number, replacing it if necessary.
    ///
    /// If the file descriptor `newfd` was previously open, it is silently closed before being reused.
    pub fn dup3(&mut self, old_fd: usize, new_fd: usize, cloexec: bool) -> LinuxResult<usize> {
        let f = self.files.get(old_fd).ok_or(LinuxError::EBADF)?.clone();
        self.files.remove(new_fd);
        self.files.add_at(new_fd, f);
        self.cloexec_bitmap.set(new_fd, cloexec);
        Ok(new_fd)
    }

    /// Duplicate the file descriptor fd using the lowest-numbered available file descriptor greater than or equal to `bound`.
    pub fn dup_with_low_bound(
        &mut self,
        fd: usize,
        bound: usize,
        cloexec: bool,
    ) -> LinuxResult<usize> {
        let f = self.files.get(fd).ok_or(LinuxError::EBADF)?.clone();
        let new_fd = self
            .files
            .add_with_low_bound(f, bound)
            .ok_or(LinuxError::EMFILE)?;
        debug_assert!(!self.cloexec_bitmap.get(new_fd));
        self.cloexec_bitmap.set(new_fd, cloexec);
        Ok(new_fd)
    }
}

impl FileSystem {
    /// Closes all file objects in the file descriptor table.
    pub fn close_all_files(&mut self) {
        for fd in 0..self.fd_table.files.capacity() {
            if self.fd_table.files.get(fd).is_some() {
                self.fd_table.files.remove(fd).unwrap();
            }
        }
        // this code might not be necessary
        self.fd_table.cloexec_bitmap = Bitmap::new();
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
        let path = mp.path;
        let vfsops = mp.fs.clone();
        let message = format!("failed to mount filesystem at {}", path);
        info!("mounting {}", path);
        root_dir.mount(path, vfsops).expect(&message);
    }

    let root_dir_arc = Arc::new(root_dir);

    let mut fs = FileSystem {
        fd_table: FdTable::default(),
        current_path: "/".into(),
        current_dir: root_dir_arc.clone(),
        root_dir: root_dir_arc.clone(),
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
pub fn absolute_path(path: &str) -> AxResult<String> {
    if path.starts_with('/') {
        Ok(axfs_vfs::path::canonicalize(path))
    } else {
        let path = current().fs.lock().as_mut().unwrap().current_path.clone() + path;
        Ok(axfs_vfs::path::canonicalize(&path))
    }
}

/// Returns the current directory.
pub fn current_dir() -> AxResult<String> {
    Ok(current().fs.lock().as_mut().unwrap().current_path.clone())
}

/// Sets the current directory.
pub fn set_current_dir(path: &str) -> AxResult {
    let mut abs_path = absolute_path(path)?;
    if !abs_path.ends_with('/') {
        abs_path += "/";
    }
    if abs_path == "/" {
        current().fs.lock().as_mut().unwrap().current_dir =
            current().fs.lock().as_mut().unwrap().root_dir.clone();
        current().fs.lock().as_mut().unwrap().current_path = "/".into();
        return Ok(());
    }

    let node = lookup(None, &abs_path)?;
    let attr = node.get_attr()?;
    if !attr.is_dir() {
        ax_err!(NotADirectory)
    } else if !attr.perm().owner_executable() {
        ax_err!(PermissionDenied)
    } else {
        current().fs.lock().as_mut().unwrap().current_dir = node;
        current().fs.lock().as_mut().unwrap().current_path = abs_path;
        Ok(())
    }
}

struct CurrentWorkingDirectoryImpl;

#[crate_interface::impl_interface]
impl CurrentWorkingDirectoryOps for CurrentWorkingDirectoryImpl {
    fn init_rootfs(mount_points: Vec<MountPoint>) {
        init_rootfs(mount_points)
    }
    fn parent_node_of(dir: Option<&VfsNodeRef>, path: &str) -> VfsNodeRef {
        parent_node_of(dir, path)
    }
    fn absolute_path(path: &str) -> AxResult<String> {
        absolute_path(path)
    }
    fn current_dir() -> AxResult<String> {
        current_dir()
    }
    fn set_current_dir(path: &str) -> AxResult {
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
