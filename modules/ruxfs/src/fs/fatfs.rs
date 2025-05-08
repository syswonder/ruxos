/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::sync::Arc;
use axerrno::ax_err;
use axfs_vfs::RelPath;
use core::cell::UnsafeCell;

use crate::dev::Disk;
use axfs_vfs::{VfsDirEntry, VfsError, VfsNodePerm, VfsResult};
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodeRef, VfsNodeType, VfsOps};
use fatfs::{Dir, File, LossyOemCpConverter, NullTimeProvider, Read, Seek, SeekFrom, Write};
use spin::RwLock;

const BLOCK_SIZE: usize = 512;

pub struct FatFileSystem {
    inner: fatfs::FileSystem<Disk, NullTimeProvider, LossyOemCpConverter>,
    root_dir: UnsafeCell<Option<VfsNodeRef>>,
}

pub struct FileWrapper<'a>(RwLock<File<'a, Disk, NullTimeProvider, LossyOemCpConverter>>);
pub struct DirWrapper<'a>(Dir<'a, Disk, NullTimeProvider, LossyOemCpConverter>);

unsafe impl Sync for FatFileSystem {}
unsafe impl Send for FatFileSystem {}
unsafe impl<'a> Send for FileWrapper<'a> {}
unsafe impl<'a> Sync for FileWrapper<'a> {}
unsafe impl<'a> Send for DirWrapper<'a> {}
unsafe impl<'a> Sync for DirWrapper<'a> {}

impl FatFileSystem {
    #[cfg(feature = "use-ramdisk")]
    pub fn new(mut disk: Disk) -> Self {
        let opts = fatfs::FormatVolumeOptions::new();
        fatfs::format_volume(&mut disk, opts).expect("failed to format volume");
        let inner = fatfs::FileSystem::new(disk, fatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem");
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    #[cfg(not(feature = "use-ramdisk"))]
    pub fn new(disk: Disk) -> Self {
        let inner = fatfs::FileSystem::new(disk, fatfs::FsOptions::new())
            .expect("failed to initialize FAT filesystem");
        Self {
            inner,
            root_dir: UnsafeCell::new(None),
        }
    }

    pub fn init(&'static self) {
        // must be called before later operations
        unsafe { *self.root_dir.get() = Some(Self::new_dir(self.inner.root_dir())) }
    }

    fn new_file(file: File<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<FileWrapper> {
        Arc::new(FileWrapper(RwLock::new(file)))
    }

    fn new_dir(dir: Dir<'_, Disk, NullTimeProvider, LossyOemCpConverter>) -> Arc<DirWrapper> {
        Arc::new(DirWrapper(dir))
    }
}

impl VfsNodeOps for FileWrapper<'static> {
    axfs_vfs::impl_vfs_non_dir_default! {}

    fn fsync(&self) -> VfsResult {
        self.0.write().flush().map_err(as_vfs_err)
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        let size = self.0.write().seek(SeekFrom::End(0)).map_err(as_vfs_err)?;
        let blocks = (size + BLOCK_SIZE as u64 - 1) / BLOCK_SIZE as u64;
        // FAT fs doesn't support permissions, we just set everything to 755
        let perm = VfsNodePerm::from_bits_truncate(0o755);

        // WARN: Inode of files, for musl dynamic linker.
        // WARN: there will be collision for files with the same size.
        // TODO: implement real inode.
        let ino = size + VfsNodeType::File as u64 | perm.bits() as u64;
        Ok(VfsNodeAttr::new(ino, perm, VfsNodeType::File, size, blocks))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let mut file = self.0.write();
        file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?;

        let mut total_read = 0;
        while total_read < buf.len() {
            let read_len = file.read(&mut buf[total_read..]).map_err(as_vfs_err)?;
            if read_len == 0 {
                break;
            }
            total_read += read_len;
        }

        Ok(total_read)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        let mut file = self.0.write();
        file.seek(SeekFrom::Start(offset)).map_err(as_vfs_err)?; // TODO: more efficient

        let mut total_write = 0;
        while total_write < buf.len() {
            let write_len = file.write(&buf[total_write..]).map_err(as_vfs_err)?;
            if write_len == 0 {
                break;
            }
            total_write += write_len;
        }

        Ok(total_write)
    }

    fn truncate(&self, size: u64) -> VfsResult {
        let mut file = self.0.write();
        file.seek(SeekFrom::Start(size)).map_err(as_vfs_err)?; // TODO: more efficient
        file.truncate().map_err(as_vfs_err)
    }
}

impl VfsNodeOps for DirWrapper<'static> {
    axfs_vfs::impl_vfs_dir_default! {}

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            1,
            VfsNodePerm::from_bits_truncate(0o755),
            VfsNodeType::Dir,
            BLOCK_SIZE as u64,
            1,
        ))
    }

    fn parent(&self) -> Option<VfsNodeRef> {
        self.0
            .open_dir("..")
            .map_or(None, |dir| Some(FatFileSystem::new_dir(dir)))
    }

    fn lookup(self: Arc<Self>, path: &RelPath) -> VfsResult<VfsNodeRef> {
        debug!("lookup at fatfs: {}", path);
        if path.is_empty() {
            return Ok(self.clone());
        }
        if let Ok(Some(is_dir)) = self.0.check_path_type(path) {
            if is_dir {
                if let Ok(dir) = self.0.open_dir(path) {
                    Ok(FatFileSystem::new_dir(dir))
                } else {
                    Err(VfsError::NotADirectory)
                }
            } else {
                if let Ok(file) = self.0.open_file(path) {
                    Ok(FatFileSystem::new_file(file))
                } else {
                    Err(VfsError::IsADirectory)
                }
            }
        } else {
            Err(VfsError::NotFound)
        }
    }

    fn create(&self, path: &RelPath, ty: VfsNodeType) -> VfsResult {
        debug!("create {:?} at fatfs: {}", ty, path);
        if path.is_empty() {
            return Ok(());
        }
        match ty {
            VfsNodeType::File => {
                self.0.create_file(path).map_err(as_vfs_err)?;
                Ok(())
            }
            VfsNodeType::Dir => {
                self.0.create_dir(path).map_err(as_vfs_err)?;
                Ok(())
            }
            _ => Err(VfsError::Unsupported),
        }
    }

    fn unlink(&self, path: &RelPath) -> VfsResult {
        debug!("remove at fatfs: {}", path);
        if path.is_empty() {
            return ax_err!(PermissionDenied);
        }
        self.0.remove(path).map_err(as_vfs_err)
    }

    fn read_dir(&self, start_idx: usize, dirents: &mut [VfsDirEntry]) -> VfsResult<usize> {
        let mut iter = self.0.iter().skip(start_idx);
        for (i, out_entry) in dirents.iter_mut().enumerate() {
            let x = iter.next();
            match x {
                Some(Ok(entry)) => {
                    let ty = if entry.is_dir() {
                        VfsNodeType::Dir
                    } else if entry.is_file() {
                        VfsNodeType::File
                    } else {
                        unreachable!()
                    };
                    *out_entry = VfsDirEntry::new(&entry.file_name(), ty);
                }
                _ => return Ok(i),
            }
        }
        Ok(dirents.len())
    }

    fn rename(&self, src_path: &RelPath, dst_path: &RelPath) -> VfsResult {
        // `src_path` and `dst_path` should in the same mounted fs
        debug!(
            "rename at fatfs, src_path: {}, dst_path: {}",
            src_path, dst_path
        );

        self.0
            .rename(src_path, &self.0, dst_path)
            .map_err(as_vfs_err)
    }
}

impl VfsOps for FatFileSystem {
    fn root_dir(&self) -> VfsNodeRef {
        let root_dir = unsafe { (*self.root_dir.get()).as_ref().unwrap() };
        root_dir.clone()
    }
}

impl fatfs::IoBase for Disk {
    type Error = ();
}

impl Read for Disk {
    fn read(&mut self, mut buf: &mut [u8]) -> Result<usize, Self::Error> {
        let mut read_len = 0;
        while !buf.is_empty() {
            match self.read_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    let tmp = buf;
                    buf = &mut tmp[n..];
                    read_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(read_len)
    }
}

impl Write for Disk {
    fn write(&mut self, mut buf: &[u8]) -> Result<usize, Self::Error> {
        let mut write_len = 0;
        while !buf.is_empty() {
            match self.write_one(buf) {
                Ok(0) => break,
                Ok(n) => {
                    buf = &buf[n..];
                    write_len += n;
                }
                Err(_) => return Err(()),
            }
        }
        Ok(write_len)
    }
    fn flush(&mut self) -> Result<(), Self::Error> {
        self.do_flush().map_err(|_| ())
    }
}

impl Seek for Disk {
    fn seek(&mut self, pos: SeekFrom) -> Result<u64, Self::Error> {
        let size = self.size();
        let new_pos = match pos {
            SeekFrom::Start(pos) => Some(pos),
            SeekFrom::Current(off) => self.position().checked_add_signed(off),
            SeekFrom::End(off) => size.checked_add_signed(off),
        }
        .ok_or(())?;
        if new_pos > size {
            warn!("Seek beyond the end of the block device");
        }
        self.set_position(new_pos);
        Ok(new_pos)
    }
}

const fn as_vfs_err(err: fatfs::Error<()>) -> VfsError {
    use fatfs::Error::*;
    match err {
        AlreadyExists => VfsError::AlreadyExists,
        CorruptedFileSystem => VfsError::InvalidData,
        DirectoryIsNotEmpty => VfsError::DirectoryNotEmpty,
        InvalidInput | InvalidFileNameLength | UnsupportedFileNameCharacter => {
            VfsError::InvalidInput
        }
        NotEnoughSpace => VfsError::StorageFull,
        NotFound => VfsError::NotFound,
        UnexpectedEof => VfsError::UnexpectedEof,
        WriteZero => VfsError::WriteZero,
        Io(_) => VfsError::Io,
        _ => VfsError::Io,
    }
}
