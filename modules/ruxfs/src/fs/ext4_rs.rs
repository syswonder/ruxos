//! Reference:
//! - ext4_rs: https://github.com/yuoo655/ext4_rs
//! - axfs: https://github.com/Starry-OS/axfs

use crate::dev::Disk;
use alloc::sync::Arc;
use alloc::vec;
use alloc::vec::*;
use axfs_vfs::{RelPath, VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use axsync::Mutex;
use core::cell::RefCell;
use ext4_rs::*;

pub struct DiskAdapter {
    inner: RefCell<Disk>,
}

unsafe impl Send for DiskAdapter {}
unsafe impl Sync for DiskAdapter {}

// The io block size of the disk layer
const DISK_BLOCK_SIZE: usize = 512;

// The block size of the file system
pub const BLOCK_SIZE: usize = 4096;

impl BlockDevice for DiskAdapter {
    fn read_offset(&self, offset: usize) -> Vec<u8> {
        let mut disk = self.inner.borrow_mut();
        let mut buf = vec![0u8; BLOCK_SIZE];

        let start_block_id = offset / DISK_BLOCK_SIZE;
        let mut offset_in_block = offset % DISK_BLOCK_SIZE;
        let mut total_bytes_read = 0;

        while total_bytes_read < buf.len() {
            let current_block_id = start_block_id + (total_bytes_read / DISK_BLOCK_SIZE);
            let bytes_to_copy =
                (buf.len() - total_bytes_read).min(DISK_BLOCK_SIZE - offset_in_block);

            let block_data = disk.read_offset(current_block_id * DISK_BLOCK_SIZE + offset_in_block);

            buf[total_bytes_read..total_bytes_read + bytes_to_copy]
                .copy_from_slice(&block_data[offset_in_block..offset_in_block + bytes_to_copy]);

            total_bytes_read += bytes_to_copy;
            offset_in_block = 0; // After the first block, subsequent blocks read from the beginning
        }

        buf
    }

    fn write_offset(&self, offset: usize, buf: &[u8]) {
        let mut disk = self.inner.borrow_mut();

        let start_block_id = offset / DISK_BLOCK_SIZE;
        let mut offset_in_block = offset % DISK_BLOCK_SIZE;

        let bytes_to_write = buf.len();
        let mut total_bytes_written = 0;

        while total_bytes_written < bytes_to_write {
            let current_block_id = start_block_id + (total_bytes_written / DISK_BLOCK_SIZE);
            let bytes_to_copy =
                (bytes_to_write - total_bytes_written).min(DISK_BLOCK_SIZE - offset_in_block);

            let mut block_data = disk.read_offset(current_block_id * DISK_BLOCK_SIZE);

            block_data[offset_in_block..offset_in_block + bytes_to_copy]
                .copy_from_slice(&buf[total_bytes_written..total_bytes_written + bytes_to_copy]);

            disk.write_offset(current_block_id * DISK_BLOCK_SIZE, &block_data)
                .unwrap();

            total_bytes_written += bytes_to_copy;
            offset_in_block = 0; // After the first block, subsequent blocks start at the beginning
        }
    }
}

pub struct Ext4FileSystem {
    #[allow(unused)]
    inner: Arc<Ext4>,
    root_dir: VfsNodeRef,
}

impl Ext4FileSystem {
    pub fn new(disk: Disk) -> Self {
        let block_device = Arc::new(DiskAdapter {
            inner: RefCell::new(disk),
        });
        let inner = Ext4::open(block_device);
        let root = Arc::new(Ext4FileWrapper::new(inner.clone()));
        Self {
            inner: inner.clone(),
            root_dir: root,
        }
    }
}

impl VfsOps for Ext4FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        Arc::clone(&self.root_dir)
    }

    fn umount(&self) -> VfsResult {
        log::info!("umount:");
        // todo!()
        Ok(())
    }
}

pub struct Ext4FileWrapper {
    ext4_file: Mutex<Ext4File>,
    ext4: Arc<Ext4>,
}

unsafe impl Send for Ext4FileWrapper {}
unsafe impl Sync for Ext4FileWrapper {}

impl Ext4FileWrapper {
    fn new(ext4: Arc<Ext4>) -> Self {
        Self {
            ext4_file: Mutex::new(Ext4File::new()),
            ext4: ext4,
        }
    }
}

impl VfsNodeOps for Ext4FileWrapper {
    /// Do something when the node is opened.
    fn open(&self) -> VfsResult {
        // log::info!("opening file");
        // let mut ext4_file = self.ext4_file.lock();
        // let r = self.ext4.ext4_open(&mut ext4_file, path, "r+", false);
        Ok(())
    }

    /// Do something when the node is closed.
    fn release(&self) -> VfsResult {
        Ok(())
    }

    /// Get the attributes of the node.
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let ext4_file = self.ext4_file.lock();
        let root_inode_ref =
            Ext4InodeRef::get_inode_ref(Arc::downgrade(&self.ext4).clone(), ext4_file.inode);
        let inode_mode = root_inode_ref.inner.inode.mode;
        let size = ext4_file.fsize;
        // BLOCK_SIZE / DISK_BLOCK_SIZE
        let blocks = root_inode_ref.inner.inode.blocks * 8;
        let (ty, perm) = map_imode(inode_mode as u16);
        drop(ext4_file);
        Ok(VfsNodeAttr::new(perm, ty, size as _, blocks as _))
    }

    // file operations:

    /// Read data from the file at the given offset.
    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut ext4_file = self.ext4_file.lock();
        ext4_file.fpos = offset as usize;

        let read_len = buf.len();
        let mut read_cnt = 0;

        let r = self
            .ext4
            .ext4_file_read(&mut ext4_file, buf, read_len, &mut read_cnt);

        if let Err(e) = r {
            match e.error() {
                Errnum::EINVAL => {
                    drop(ext4_file);
                    Ok(0)
                }
                _ => {
                    drop(ext4_file);
                    Err(VfsError::InvalidInput)
                }
            }
        } else {
            drop(ext4_file);
            Ok(read_len)
        }
    }

    /// Write data to the file at the given offset.
    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut ext4_file = self.ext4_file.lock();
        ext4_file.fpos = offset as usize;

        let write_size = buf.len();

        self.ext4.ext4_file_write(&mut ext4_file, &buf, write_size);

        Ok(write_size)
    }

    /// Flush the file, synchronize the data to disk.
    fn fsync(&self) -> VfsResult {
        todo!()
    }

    /// Truncate the file to the given size.
    fn truncate(&self, _size: u64) -> VfsResult {
        todo!()
    }

    // directory operations:

    /// Get the parent directory of this directory.
    ///
    /// Return `None` if the node is a file.
    fn parent(&self) -> Option<VfsNodeRef> {
        None
    }

    /// Lookup the node with given `path` in the directory.
    ///
    /// Return the node if found.
    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        let mut ext4_file = self.ext4_file.lock();
        let r = self.ext4.ext4_open(&mut ext4_file, path, "r+", false);

        if let Err(e) = r {
            match e.error() {
                Errnum::ENOENT => Err(VfsError::NotFound),
                Errnum::EALLOCFIAL => Err(VfsError::InvalidInput),
                Errnum::ELINKFIAL => Err(VfsError::InvalidInput),

                _ => Err(VfsError::InvalidInput),
            }
        } else {
            drop(ext4_file);
            // log::error!("file found");
            Ok(self.clone())
        }
    }

    /// Create a new node with the given `path` in the directory
    ///
    /// Return [`Ok(())`](Ok) if it already exists.
    fn create(&self, path: &RelPath, ty: VfsNodeType) -> VfsResult {
        let types = match ty {
            VfsNodeType::Fifo => DirEntryType::EXT4_DE_FIFO,
            VfsNodeType::CharDevice => DirEntryType::EXT4_DE_CHRDEV,
            VfsNodeType::Dir => DirEntryType::EXT4_DE_DIR,
            VfsNodeType::BlockDevice => DirEntryType::EXT4_DE_BLKDEV,
            VfsNodeType::File => DirEntryType::EXT4_DE_REG_FILE,
            VfsNodeType::SymLink => DirEntryType::EXT4_DE_SYMLINK,
            VfsNodeType::Socket => DirEntryType::EXT4_DE_SOCK,
        };

        let mut ext4file = self.ext4_file.lock();

        if types == DirEntryType::EXT4_DE_DIR {
            let _ = self.ext4.ext4_dir_mk(path);
        } else {
            let _ = self.ext4.ext4_open(&mut ext4file, path, "w+", true);
        }

        drop(ext4file);

        Ok(())
    }

    /// Remove the node with the given `path` in the directory.
    fn unlink(&self, _path: &RelPath) -> VfsResult {
        todo!()
    }

    /// Read directory entries into `dirents`, starting from `start_idx`.
    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let ext4_file = self.ext4_file.lock();
        let inode_num = ext4_file.inode;
        let entries: Vec<Ext4DirEntry> = self.ext4.read_dir_entry(inode_num as _);
        let mut iter = entries.into_iter().skip(start_idx);

        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let x: Option<Ext4DirEntry> = iter.next();
            match x {
                Some(ext4direntry) => {
                    let name = ext4direntry.name;
                    let name_len = ext4direntry.name_len;
                    let file_type = unsafe { ext4direntry.inner.inode_type };
                    let (ty, _) = map_dir_imode(file_type as u16);
                    let name = get_name(name, name_len as usize).unwrap();
                    *out_entry = VfsDirEntry::new(name.as_str(), ty);
                }
                _ => return Ok(i),
            }
        }

        drop(ext4_file);
        Ok(dirents.len())
    }

    /// Renames or moves existing file or directory.
    fn rename(&self, _src_path: &RelPath, _dst_path: &RelPath) -> VfsResult {
        todo!()
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self as &dyn core::any::Any
    }
}

fn map_dir_imode(imode: u16) -> (VfsNodeType, VfsNodePerm) {
    let diren_type = imode;
    let type_code = ext4_rs::DirEntryType::from_bits(diren_type as u8).unwrap();
    let ty = match type_code {
        DirEntryType::EXT4_DE_REG_FILE => VfsNodeType::File,
        DirEntryType::EXT4_DE_DIR => VfsNodeType::Dir,
        DirEntryType::EXT4_DE_CHRDEV => VfsNodeType::CharDevice,
        DirEntryType::EXT4_DE_BLKDEV => VfsNodeType::BlockDevice,
        DirEntryType::EXT4_DE_FIFO => VfsNodeType::Fifo,
        DirEntryType::EXT4_DE_SOCK => VfsNodeType::Socket,
        DirEntryType::EXT4_DE_SYMLINK => VfsNodeType::SymLink,
        _ => {
            // log::info!("{:x?}", imode);
            VfsNodeType::File
        }
    };

    let perm = ext4_rs::FileMode::from_bits_truncate(imode);
    let mut vfs_perm = VfsNodePerm::from_bits_truncate(0);

    if perm.contains(ext4_rs::FileMode::S_IXOTH) {
        vfs_perm |= VfsNodePerm::OTHER_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWOTH) {
        vfs_perm |= VfsNodePerm::OTHER_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IROTH) {
        vfs_perm |= VfsNodePerm::OTHER_READ;
    }

    if perm.contains(ext4_rs::FileMode::S_IXGRP) {
        vfs_perm |= VfsNodePerm::GROUP_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWGRP) {
        vfs_perm |= VfsNodePerm::GROUP_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IRGRP) {
        vfs_perm |= VfsNodePerm::GROUP_READ;
    }

    if perm.contains(ext4_rs::FileMode::S_IXUSR) {
        vfs_perm |= VfsNodePerm::OWNER_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWUSR) {
        vfs_perm |= VfsNodePerm::OWNER_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IRUSR) {
        vfs_perm |= VfsNodePerm::OWNER_READ;
    }

    (ty, vfs_perm)
}

fn map_imode(imode: u16) -> (VfsNodeType, VfsNodePerm) {
    let file_type = (imode & 0xf000) as usize;
    let ty = match file_type {
        EXT4_INODE_MODE_FIFO => VfsNodeType::Fifo,
        EXT4_INODE_MODE_CHARDEV => VfsNodeType::CharDevice,
        EXT4_INODE_MODE_DIRECTORY => VfsNodeType::Dir,
        EXT4_INODE_MODE_BLOCKDEV => VfsNodeType::BlockDevice,
        EXT4_INODE_MODE_FILE => VfsNodeType::File,
        EXT4_INODE_MODE_SOFTLINK => VfsNodeType::SymLink,
        EXT4_INODE_MODE_SOCKET => VfsNodeType::Socket,
        _ => {
            // log::info!("{:x?}", imode);
            VfsNodeType::File
        }
    };

    let perm = ext4_rs::FileMode::from_bits_truncate(imode);
    let mut vfs_perm = VfsNodePerm::from_bits_truncate(0);

    if perm.contains(ext4_rs::FileMode::S_IXOTH) {
        vfs_perm |= VfsNodePerm::OTHER_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWOTH) {
        vfs_perm |= VfsNodePerm::OTHER_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IROTH) {
        vfs_perm |= VfsNodePerm::OTHER_READ;
    }

    if perm.contains(ext4_rs::FileMode::S_IXGRP) {
        vfs_perm |= VfsNodePerm::GROUP_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWGRP) {
        vfs_perm |= VfsNodePerm::GROUP_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IRGRP) {
        vfs_perm |= VfsNodePerm::GROUP_READ;
    }

    if perm.contains(ext4_rs::FileMode::S_IXUSR) {
        vfs_perm |= VfsNodePerm::OWNER_EXEC;
    }
    if perm.contains(ext4_rs::FileMode::S_IWUSR) {
        vfs_perm |= VfsNodePerm::OWNER_WRITE;
    }
    if perm.contains(ext4_rs::FileMode::S_IRUSR) {
        vfs_perm |= VfsNodePerm::OWNER_READ;
    }

    (ty, vfs_perm)
}
