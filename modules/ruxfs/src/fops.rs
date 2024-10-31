/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Low-level filesystem operations.

use axerrno::{ax_err, ax_err_type, AxResult};
use axfs_vfs::path::{AbsPath, RelPath};
use axfs_vfs::{VfsNodeRef, VfsNodeType, VfsNodeOps};
use axio::SeekFrom;
use capability::{Cap, WithCap};

use crate::root::{CURRENT_DIR, CURRENT_DIR_PATH, ROOT_DIR};

#[cfg(feature = "myfs")]
pub use crate::dev::Disk;
#[cfg(feature = "myfs")]
pub use crate::fs::myfs::MyFileSystemIf;

/// Alias of [`axfs_vfs::VfsNodeType`].
pub type FileType = axfs_vfs::VfsNodeType;
/// Alias of [`axfs_vfs::VfsDirEntry`].
pub type DirEntry = axfs_vfs::VfsDirEntry;
/// Alias of [`axfs_vfs::VfsNodeAttr`].
pub type FileAttr = axfs_vfs::VfsNodeAttr;
/// Alias of [`axfs_vfs::VfsNodePerm`].
pub type FilePerm = axfs_vfs::VfsNodePerm;

/// An opened file object, with open permissions and a cursor.
pub struct File {
    node: WithCap<VfsNodeRef>,
    is_append: bool,
    offset: u64,
}

/// An opened directory object, with open permissions and a cursor for
/// [`read_dir`](Directory::read_dir).
pub struct Directory {
    node: WithCap<VfsNodeRef>,
    entry_idx: usize,
}

/// Options and flags which can be used to configure how a file is opened.
#[derive(Clone)]
pub struct OpenOptions {
    // generic
    pub read: bool,
    pub write: bool,
    pub append: bool,
    pub truncate: bool,
    pub create: bool,
    pub create_new: bool,
    // system-specific
    _custom_flags: i32,
    _mode: u32,
}

impl OpenOptions {
    /// Creates a blank new set of options ready for configuration.
    pub const fn new() -> Self {
        Self {
            // generic
            read: false,
            write: false,
            append: false,
            truncate: false,
            create: false,
            create_new: false,
            // system-specific
            _custom_flags: 0,
            _mode: 0o666,
        }
    }
    /// Sets the option for read access.
    pub fn read(&mut self, read: bool) {
        self.read = read;
    }
    /// Sets the option for write access.
    pub fn write(&mut self, write: bool) {
        self.write = write;
    }
    /// Sets the option for the append mode.
    pub fn append(&mut self, append: bool) {
        self.append = append;
    }
    /// Sets the option for truncating a previous file.
    pub fn truncate(&mut self, truncate: bool) {
        self.truncate = truncate;
    }
    /// Sets the option to create a new file, or open it if it already exists.
    pub fn create(&mut self, create: bool) {
        self.create = create;
    }
    /// Sets the option to create a new file, failing if it already exists.
    pub fn create_new(&mut self, create_new: bool) {
        self.create_new = create_new;
    }

    pub const fn is_valid(&self) -> bool {
        if !self.read && !self.write && !self.append {
            return false;
        }
        match (self.write, self.append) {
            (true, false) => {}
            (false, false) => {
                if self.truncate || self.create || self.create_new {
                    return false;
                }
            }
            (_, true) => {
                if self.truncate && !self.create_new {
                    return false;
                }
            }
        }
        true
    }
}

impl File {
    /// Create an opened file.
    pub fn new(node: VfsNodeRef, cap: Cap, is_append: bool) -> Self {
        Self {
            node: WithCap::new(node, cap),
            offset: 0,
            is_append,
        }
    }

    /// Truncates the file to the specified size.
    pub fn truncate(&self, size: u64) -> AxResult {
        self.node.access(Cap::WRITE)?.truncate(size)?;
        Ok(())
    }

    /// Reads the file at the current position. Returns the number of bytes
    /// read.
    ///
    /// After the read, the cursor will be advanced by the number of bytes read.
    pub fn read(&mut self, buf: &mut [u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::READ)?;
        let read_len = node.read_at(self.offset, buf)?;
        self.offset += read_len as u64;
        Ok(read_len)
    }

    /// Reads the file at the given position. Returns the number of bytes read.
    ///
    /// It does not update the file cursor.
    pub fn read_at(&self, offset: u64, buf: &mut [u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::READ)?;
        let read_len = node.read_at(offset, buf)?;
        Ok(read_len)
    }

    /// Writes the file at the current position. Returns the number of bytes
    /// written.
    ///
    /// After the write, the cursor will be advanced by the number of bytes
    /// written.
    pub fn write(&mut self, buf: &[u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::WRITE)?;
        if self.is_append {
            self.offset = self.get_attr()?.size();
        };
        let write_len = node.write_at(self.offset, buf)?;
        self.offset += write_len as u64;
        Ok(write_len)
    }

    /// Writes the file at the given position. Returns the number of bytes
    /// written.
    ///
    /// It does not update the file cursor.
    pub fn write_at(&self, offset: u64, buf: &[u8]) -> AxResult<usize> {
        let node = self.node.access(Cap::WRITE)?;
        let write_len = node.write_at(offset, buf)?;
        Ok(write_len)
    }

    /// Flushes the file, writes all buffered data to the underlying device.
    pub fn flush(&self) -> AxResult {
        self.node.access(Cap::WRITE)?.fsync()?;
        Ok(())
    }

    /// Sets the cursor of the file to the specified offset. Returns the new
    /// position after the seek.
    pub fn seek(&mut self, pos: SeekFrom) -> AxResult<u64> {
        let size = self.get_attr()?.size();
        let new_offset = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.offset.checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or_else(|| ax_err_type!(InvalidInput))?;
        self.offset = new_offset;
        Ok(new_offset)
    }

    /// Gets the file attributes.
    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.access(Cap::empty())?.get_attr()
    }
}

impl Directory {
    /// Access the underlying `VfsNode`
    fn access_node(&self) -> AxResult<&VfsNodeRef> {
        self.node.access(Cap::EXECUTE).or(ax_err!(PermissionDenied))
    }

    /// Creates an opened directory.
    pub fn new(node: VfsNodeRef, cap: Cap) -> Self {
        Self {
            node: WithCap::new(node, cap),
            entry_idx: 0,
        }
    }

    /// Gets the file attributes.
    pub fn get_attr(&self) -> AxResult<FileAttr> {
        self.node.access(Cap::empty())?.get_attr()
    }

    /// Looks up a child node by name.
    pub fn lookup(&self, path: &RelPath) -> AxResult<VfsNodeRef> {
        self.access_node()?.clone().lookup(path)
    }

    /// Gets the file attributes of the file at the path relative to this directory.
    /// Returns a [`FileAttr`] object.
    pub fn get_child_attr(&self, path: &RelPath) -> AxResult<FileAttr> {
        let node = self.lookup(path)?;
        node.get_attr()
    }

    /// Opens a node as a directory, with permission checked.
    pub fn open_dir(&self, node: VfsNodeRef, cap: Cap) -> AxResult<Directory> {
        let attr = node.get_attr()?;
        if !perm_to_cap(attr.perm()).contains(cap) {
            return ax_err!(PermissionDenied);
        }
        node.open()?;
        Ok(Self::new(node, cap | Cap::EXECUTE))
    }

    /// Opens a node as a file, with permission checked.
    pub fn open_file(&self, node: VfsNodeRef, cap: Cap, append: bool) -> AxResult<File> {
        let attr = node.get_attr()?;
        if !perm_to_cap(attr.perm()).contains(cap) {
            return ax_err!(PermissionDenied);
        }
        node.open()?;
        Ok(File::new(node, cap, append))
    }

    /// Creates an empty file at the path relative to this directory.
    ///
    /// This function will not check if the path exists, check it with
    /// [`lookup`] first.
    pub fn create_file(&self, path: &RelPath) -> AxResult {
        self.access_node()?.clone().create(path, VfsNodeType::File)
    }

    /// Creates an empty directory at the path relative to this directory.
    ///
    /// This function will not check if the path exists, check it with
    /// [`lookup`] first.
    pub fn create_dir(&self, path: &RelPath) -> AxResult {
        self.access_node()?.clone().create(path, VfsNodeType::Dir)
    }

    /// Removes a file (or directory) at the path relative to this directory.
    ///
    /// This function will not check if the file (or directory) exits or removeable,
    /// check it with [`lookup`] first.
    pub fn remove(&self, path: &RelPath) -> AxResult {
        self.access_node()?.remove(path)
    }

    /// Rename a file or directory to a new name. This only works then the new path
    /// is in the same mounted fs.
    ///
    /// This function will not check if the old path or new path exists, check it with
    /// [`lookup`] first.
    pub fn rename(&self, old: &RelPath, new: &RelPath) -> AxResult {
        self.access_node()?.rename(old, new)
    }

    /// Reads directory entries starts from the current position into the
    /// given buffer. Returns the number of entries read.
    ///
    /// After the read, the cursor will be advanced by the number of entries
    /// read.
    pub fn read_dir(&mut self, dirents: &mut [DirEntry]) -> AxResult<usize> {
        let n = self
            .node
            .access(Cap::READ)?
            .read_dir(self.entry_idx, dirents)?;
        self.entry_idx += n;
        Ok(n)
    }

    /// Get the entry cursor of the directory.
    pub fn entry_idx(&self) -> usize {
        self.entry_idx
    }

    /// Set the entry cursor of the directory.
    pub fn set_entry_idx(&mut self, idx: usize) {
        self.entry_idx = idx;
    }
}

/* File operations with absolute path */

/// Look up a file given an absolute path.
pub fn lookup(path: &AbsPath) -> AxResult<VfsNodeRef> {
    ROOT_DIR.clone().lookup(&path.to_rel())
}

/// Get the file attributes given an absolute path.
pub fn get_attr(path: &AbsPath) -> AxResult<FileAttr> {
    let node = lookup(path)?;
    node.get_attr()
}

/// Open a node as a file, with permission checked.
pub fn open_file(node: VfsNodeRef, cap: Cap, append: bool) -> AxResult<File> {
    let attr = node.get_attr()?;
    if !perm_to_cap(attr.perm()).contains(cap) {
        return ax_err!(PermissionDenied);
    }
    node.open()?;
    Ok(File::new(node, cap, append))
}

/// Open a node as a directory, with permission checked.
pub fn open_dir(node: VfsNodeRef, cap: Cap) -> AxResult<Directory> {
    let attr = node.get_attr()?;
    if !perm_to_cap(attr.perm()).contains(cap) {
        return ax_err!(PermissionDenied);
    }
    node.open()?;
    Ok(Directory::new(node, cap | Cap::EXECUTE))
}

/// Create a file given an absolute path.
///
/// This function will not check if the file exists, check it with [`lookup`] first.
pub fn create_file(path: &AbsPath) -> AxResult {
    ROOT_DIR.create(&path.to_rel(), VfsNodeType::File)
}

/// Create a directory given an absolute path.
///
/// This function will not check if the directory exists, check it with [`lookup`] first.
pub fn create_dir(path: &AbsPath) -> AxResult {
    ROOT_DIR.create(&path.to_rel(), VfsNodeType::Dir)
}

/// Create a directory recursively given an absolute path.
///
/// This function will not check if the directory exists, check it with [`lookup`] first.
pub fn create_dir_all(path: &AbsPath) -> AxResult {
    ROOT_DIR.create_recursive(&path.to_rel(), VfsNodeType::Dir)
}

/// Remove a file given an absolute path.
///
/// This function will not check if the file exits or removeable,
/// check it with [`lookup`] first.
pub fn remove_file(path: &AbsPath) -> AxResult {
    ROOT_DIR.remove(&path.to_rel())
}

/// Remove a directory given an absolute path.
///
/// This function will not check if the directory exists or is empty,
/// check it with [`lookup`] first.
pub fn remove_dir(path: &AbsPath) -> AxResult {
    if ROOT_DIR.contains(path) {
        return ax_err!(PermissionDenied);
    }
    ROOT_DIR.remove(&path.to_rel())
}

/// Rename a file given an old and a new absolute path.
///
/// This function will not check if the old path or new path exists, check it with
/// [`lookup`] first.
pub fn rename(old: &AbsPath, new: &AbsPath) -> AxResult {
    ROOT_DIR.rename(&old.to_rel(), &new.to_rel())
}

/// Get current working directory.
pub fn current_dir<'a>() -> AbsPath<'a> {
    CURRENT_DIR_PATH.lock().clone()
}

/// Concatenate a path to the current working directory.
pub fn concat_path(path: &RelPath) -> AbsPath<'static> {
    current_dir().join(path)
}

/// Set current working directory.
///
/// Returns error if the path does not exist or is not a directory.
pub fn set_current_dir(path: AbsPath<'static>) -> AxResult {
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

impl Drop for File {
    fn drop(&mut self) {
        unsafe { self.node.access_unchecked().release().ok() };
    }
}

impl Drop for Directory {
    fn drop(&mut self) {
        unsafe { self.node.access_unchecked().release().ok() };
    }
}

pub fn perm_to_cap(perm: FilePerm) -> Cap {
    let mut cap = Cap::empty();
    if perm.owner_readable() {
        cap |= Cap::READ;
    }
    if perm.owner_writable() {
        cap |= Cap::WRITE;
    }
    if perm.owner_executable() {
        cap |= Cap::EXECUTE;
    }
    cap
}
