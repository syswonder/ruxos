//! Reference:
//! - another_ext4: https://github.com/LJxTHUCS/another_ext4
//! - axfs: https://github.com/Starry-OS/axfs

use crate::dev::Disk;
use alloc::sync::Arc;
use another_ext4::{
    Block, BlockDevice, ErrCode as Ext4ErrorCode, Ext4, Ext4Error, FileType as EXt4FileType,
    InodeMode as Ext4InodeMode, BLOCK_SIZE as EXT4_BLOCK_SIZE, EXT4_ROOT_INO,
};
use axfs_vfs::{RelPath, VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use axsync::Mutex;

pub struct DiskAdapter(Arc<Mutex<Disk>>);

unsafe impl Send for DiskAdapter {}
unsafe impl Sync for DiskAdapter {}

// The io block size of the disk layer
const DISK_BLOCK_SIZE: usize = 512;

// The block size of the file system
pub const BLOCK_SIZE: usize = EXT4_BLOCK_SIZE;

impl BlockDevice for DiskAdapter {
    fn read_block(&self, block_id: u64) -> Block {
        let mut disk = self.0.lock();
        let base = block_id as usize * EXT4_BLOCK_SIZE;
        let mut data = [0u8; EXT4_BLOCK_SIZE];
        // Per-disk-block read
        for i in 0..(EXT4_BLOCK_SIZE / DISK_BLOCK_SIZE) {
            let dblock = disk.read_offset(base + i * DISK_BLOCK_SIZE);
            data[i * DISK_BLOCK_SIZE..(i + 1) * DISK_BLOCK_SIZE].copy_from_slice(&dblock);
        }
        Block::new(block_id, data)
    }

    fn write_block(&self, block: &Block) {
        let mut disk = self.0.lock();
        let base = block.id as usize * EXT4_BLOCK_SIZE;
        // Per-disk-block write
        for i in 0..(EXT4_BLOCK_SIZE / DISK_BLOCK_SIZE) {
            let dblock = &block.data[i * DISK_BLOCK_SIZE..(i + 1) * DISK_BLOCK_SIZE];
            let _ = disk.write_offset(base + i * DISK_BLOCK_SIZE, dblock);
        }
    }
}

pub struct Ext4FileSystem(Arc<Ext4>);

impl Ext4FileSystem {
    pub fn new(disk: Disk) -> Self {
        let block_device = Arc::new(DiskAdapter(Arc::new(Mutex::new(disk))));
        let ext4 = Ext4::load(block_device).expect("Failed to load ext4 filesystem");
        log::info!("Ext4 filesystem loaded");
        Self(Arc::new(ext4))
    }
}

impl VfsOps for Ext4FileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        Arc::new(Ext4VirtInode::new(EXT4_ROOT_INO, self.0.clone()))
    }
    fn umount(&self) -> VfsResult {
        self.0.flush_all();
        Ok(())
    }
}

pub struct Ext4VirtInode {
    id: u32,
    fs: Arc<Ext4>,
}

unsafe impl Send for Ext4VirtInode {}
unsafe impl Sync for Ext4VirtInode {}

impl Ext4VirtInode {
    fn new(id: u32, fs: Arc<Ext4>) -> Self {
        log::trace!("Create Ext4VirtInode {}", id);
        Self { id, fs }
    }
}

impl VfsNodeOps for Ext4VirtInode {
    fn open(&self) -> VfsResult {
        Ok(())
    }

    fn release(&self) -> VfsResult {
        Ok(())
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        self.fs
            .getattr(self.id)
            .map(|attr| {
                VfsNodeAttr::new(
                    map_perm(attr.perm),
                    map_type(attr.ftype),
                    attr.size,
                    attr.blocks,
                )
            })
            .map_err(map_error)
    }

    // file operations:

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        self.fs
            .read(self.id, offset as usize, buf)
            .map_err(map_error)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        self.fs
            .write(self.id, offset as usize, buf)
            .map_err(map_error)
    }

    fn fsync(&self) -> VfsResult {
        Ok(())
    }

    fn truncate(&self, size: u64) -> VfsResult {
        // TODO: Simple implementation, just set the size,
        // not truncate the file in the disk
        self.fs
            .setattr(
                self.id,
                None,
                None,
                None,
                Some(size),
                None,
                None,
                None,
                None,
            )
            .map_err(map_error)
    }

    // directory operations:

    fn parent(&self) -> Option<VfsNodeRef> {
        self.fs.lookup(self.id, "..").map_or(None, |parent| {
            Some(Arc::new(Ext4VirtInode::new(parent, self.fs.clone())))
        })
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        match self.fs.generic_lookup(self.id, path) {
            Ok(id) => Ok(Arc::new(Ext4VirtInode::new(id, self.fs.clone()))),
            Err(e) => Err(map_error(e)),
        }
    }

    fn create(&self, path: &RelPath, ty: VfsNodeType) -> VfsResult {
        if self.fs.generic_lookup(self.id, path).is_ok() {
            return Ok(());
        }
        let mode = Ext4InodeMode::from_type_and_perm(map_type_inv(ty), Ext4InodeMode::ALL_RWX);
        self.fs
            .generic_create(self.id, path, mode)
            .map(|_| ())
            .map_err(map_error)
    }

    fn unlink(&self, path: &RelPath) -> VfsResult {
        self.fs.unlink(self.id, path).map_err(map_error)
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        self.fs
            .listdir(self.id)
            .map(|entries| {
                for (i, entry) in entries.iter().skip(start_idx).enumerate() {
                    if i >= dirents.len() {
                        return i;
                    }
                    dirents[i] = VfsDirEntry::new(&entry.name(), map_type(entry.file_type()));
                }
                entries.len() - start_idx
            })
            .map_err(map_error)
    }

    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult {
        self.fs
            .generic_rename(self.id, src_path, dst_path)
            .map_err(map_error)
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self as &dyn core::any::Any
    }
}

fn map_error(ext4_err: Ext4Error) -> VfsError {
    log::warn!("Ext4 error: {:?}", ext4_err);
    match ext4_err.code() {
        Ext4ErrorCode::EPERM => VfsError::PermissionDenied,
        Ext4ErrorCode::ENOENT => VfsError::NotFound,
        Ext4ErrorCode::EIO => VfsError::Io,
        Ext4ErrorCode::ENXIO => VfsError::Io, // ?
        Ext4ErrorCode::E2BIG => VfsError::InvalidInput,
        Ext4ErrorCode::ENOMEM => VfsError::NoMemory,
        Ext4ErrorCode::EACCES => VfsError::PermissionDenied, // ?
        Ext4ErrorCode::EFAULT => VfsError::BadAddress,
        Ext4ErrorCode::EEXIST => VfsError::AlreadyExists,
        Ext4ErrorCode::ENODEV => VfsError::Io, // ?
        Ext4ErrorCode::ENOTDIR => VfsError::NotADirectory,
        Ext4ErrorCode::EISDIR => VfsError::IsADirectory,
        Ext4ErrorCode::EINVAL => VfsError::InvalidData,
        Ext4ErrorCode::EFBIG => VfsError::InvalidData,
        Ext4ErrorCode::ENOSPC => VfsError::StorageFull,
        Ext4ErrorCode::EROFS => VfsError::PermissionDenied,
        Ext4ErrorCode::EMLINK => VfsError::Io, // ?
        Ext4ErrorCode::ERANGE => VfsError::InvalidData,
        Ext4ErrorCode::ENOTEMPTY => VfsError::DirectoryNotEmpty,
        Ext4ErrorCode::ENODATA => VfsError::NotFound, // `NotFound` only for entry?
        Ext4ErrorCode::ENOTSUP => VfsError::Io,       // ?
        Ext4ErrorCode::ELINKFAIL => VfsError::Io,     // ?
        Ext4ErrorCode::EALLOCFAIL => VfsError::StorageFull, // ?
    }
}

fn map_type(ext4_type: EXt4FileType) -> VfsNodeType {
    match ext4_type {
        EXt4FileType::RegularFile => VfsNodeType::File,
        EXt4FileType::Directory => VfsNodeType::Dir,
        EXt4FileType::CharacterDev => VfsNodeType::CharDevice,
        EXt4FileType::BlockDev => VfsNodeType::BlockDevice,
        EXt4FileType::Fifo => VfsNodeType::Fifo,
        EXt4FileType::Socket => VfsNodeType::Socket,
        EXt4FileType::SymLink => VfsNodeType::SymLink,
        EXt4FileType::Unknown => VfsNodeType::File,
    }
}

fn map_type_inv(vfs_type: VfsNodeType) -> EXt4FileType {
    match vfs_type {
        VfsNodeType::File => EXt4FileType::RegularFile,
        VfsNodeType::Dir => EXt4FileType::Directory,
        VfsNodeType::CharDevice => EXt4FileType::CharacterDev,
        VfsNodeType::BlockDevice => EXt4FileType::BlockDev,
        VfsNodeType::Fifo => EXt4FileType::Fifo,
        VfsNodeType::Socket => EXt4FileType::Socket,
        VfsNodeType::SymLink => EXt4FileType::SymLink,
    }
}

fn map_perm(perm: Ext4InodeMode) -> VfsNodePerm {
    let mut vfs_perm = VfsNodePerm::from_bits_truncate(0);
    if perm.contains(Ext4InodeMode::USER_READ) {
        vfs_perm |= VfsNodePerm::OWNER_READ;
    }
    if perm.contains(Ext4InodeMode::USER_WRITE) {
        vfs_perm |= VfsNodePerm::OWNER_WRITE;
    }
    if perm.contains(Ext4InodeMode::USER_EXEC) {
        vfs_perm |= VfsNodePerm::OWNER_EXEC;
    }
    if perm.contains(Ext4InodeMode::GROUP_READ) {
        vfs_perm |= VfsNodePerm::GROUP_READ;
    }
    if perm.contains(Ext4InodeMode::GROUP_WRITE) {
        vfs_perm |= VfsNodePerm::GROUP_WRITE;
    }
    if perm.contains(Ext4InodeMode::GROUP_EXEC) {
        vfs_perm |= VfsNodePerm::GROUP_EXEC;
    }
    if perm.contains(Ext4InodeMode::OTHER_READ) {
        vfs_perm |= VfsNodePerm::OTHER_READ;
    }
    if perm.contains(Ext4InodeMode::OTHER_WRITE) {
        vfs_perm |= VfsNodePerm::OTHER_WRITE;
    }
    if perm.contains(Ext4InodeMode::OTHER_EXEC) {
        vfs_perm |= VfsNodePerm::OTHER_EXEC;
    }
    vfs_perm
}
