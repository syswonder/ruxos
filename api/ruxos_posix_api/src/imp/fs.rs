/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{borrow::Cow, string::String, sync::Arc};
use core::ffi::{c_char, c_int, c_long, c_void, CStr};

use axerrno::{LinuxError, LinuxResult};
use axio::{PollState, SeekFrom};
use axsync::Mutex;
use ruxfdtable::{FileLike, RuxStat};
use ruxfs::{
    api::set_current_dir,
    fops::{DirEntry, OpenOptions},
};

use super::fd_ops::get_file_like;
use crate::ctypes;
use alloc::vec::Vec;

pub struct File {
    pub(crate) inner: Mutex<ruxfs::fops::File>,
}

impl File {
    pub(crate) fn new(inner: ruxfs::fops::File) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    pub(crate) fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    pub(crate) fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }
}

impl FileLike for File {
    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().read(buf)?)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        Ok(self.inner.lock().write(buf)?)
    }

    fn flush(&self) -> LinuxResult {
        Ok(self.inner.lock().flush()?)
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        let metadata = self.inner.lock().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;

        // Inode of files, for musl dynamic linker.
        // WARN: there will be collision for files with the same size.
        // TODO: implement real inode.
        let st_ino = metadata.size() + st_mode as u64;

        let res = RuxStat::from(ctypes::stat {
            st_ino,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        });

        Ok(res)
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

pub struct Directory {
    inner: Mutex<ruxfs::fops::Directory>,
}

impl Directory {
    fn new(inner: ruxfs::fops::Directory) -> Self {
        Self {
            inner: Mutex::new(inner),
        }
    }

    fn add_to_fd_table(self) -> LinuxResult<c_int> {
        super::fd_ops::add_file_like(Arc::new(self))
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = super::fd_ops::get_file_like(fd)?;
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
        let metadata = self.inner.lock().get_attr()?;
        let ty = metadata.file_type() as u8;
        let perm = metadata.perm().bits() as u32;
        let st_mode = ((ty as u32) << 12) | perm;
        Ok(RuxStat::from(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_size: metadata.size() as _,
            st_blocks: metadata.blocks() as _,
            st_blksize: 512,
            ..Default::default()
        }))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        Ok(PollState {
            readable: true,
            writable: true,
        })
    }

    fn set_nonblocking(&self, _nonblocking: bool) -> LinuxResult {
        Ok(())
    }
}

/// Convert open flags to [`OpenOptions`].
fn flags_to_options(flags: c_int, _mode: ctypes::mode_t) -> OpenOptions {
    let flags = flags as u32;
    let mut options = OpenOptions::new();
    match flags & 0b11 {
        ctypes::O_RDONLY => options.read(true),
        ctypes::O_WRONLY => options.write(true),
        _ => {
            options.read(true);
            options.write(true);
        }
    };
    if flags & ctypes::O_APPEND != 0 {
        options.append(true);
    }
    if flags & ctypes::O_TRUNC != 0 {
        options.truncate(true);
    }
    if flags & ctypes::O_CREAT != 0 {
        options.create(true);
    }
    if flags & ctypes::O_EXEC != 0 {
        options.create_new(true);
    }
    options
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
pub fn sys_open(filename: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    let filename = char_ptr_to_absolute_path(filename);
    debug!("sys_open <= {:?} {:#o} {:#o}", filename, flags, mode);
    syscall_body!(sys_open, {
        let options = flags_to_options(flags, mode);
        let file = ruxfs::fops::File::open(&filename?, &options)?;
        File::new(file).add_to_fd_table()
    })
}

/// Open a file under a specific dir
pub fn sys_openat(fd: usize, path: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    let path = char_ptr_to_absolute_path(path);
    let fd: c_int = fd as c_int;
    debug!("sys_openat <= {}, {:?}, {:#o} {:#o}", fd, path, flags, mode);
    syscall_body!(sys_openat, {
        let options = flags_to_options(flags, mode);
        if (flags as u32) & ctypes::O_DIRECTORY != 0 {
            let dir = if fd == ctypes::AT_FDCWD {
                ruxfs::fops::Directory::open_dir(&path?, &options)?
            } else {
                Directory::from_fd(fd)?
                    .inner
                    .lock()
                    .open_dir_at(&path?, &options)?
            };
            Directory::new(dir).add_to_fd_table()
        } else {
            let file = if fd == ctypes::AT_FDCWD {
                ruxfs::fops::File::open(&path?, &options)?
            } else {
                Directory::from_fd(fd)?
                    .inner
                    .lock()
                    .open_file_at(&path?, &options)?
            };
            File::new(file).add_to_fd_table()
        }
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Read data from a file at a specific offset.
pub fn sys_pread64(
    fd: c_int,
    buf: *mut c_void,
    count: usize,
    pos: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!("sys_pread64 <= {} {} {}", fd, count, pos);
    syscall_body!(sys_pread64, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
        let size = File::from_fd(fd)?.inner.lock().read_at(pos as u64, dst)?;
        Ok(size as ctypes::ssize_t)
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Write data from a file at a specific offset.
pub fn sys_pwrite64(
    fd: c_int,
    buf: *const c_void,
    count: usize,
    pos: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!("sys_pwrite64 <= {} {} {}", fd, count, pos);
    syscall_body!(sys_pwrite64, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let src = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
        let size = File::from_fd(fd)?.inner.lock().write_at(pos as u64, src)?;
        Ok(size as ctypes::ssize_t)
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
pub fn sys_lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    debug!("sys_lseek <= {} {} {}", fd, offset, whence);
    syscall_body!(sys_lseek, {
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = File::from_fd(fd)?.inner.lock().seek(pos)?;
        Ok(off)
    })
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fsync(fd: c_int) -> c_int {
    debug!("sys_fsync <= fd: {}", fd);
    syscall_body!(sys_fsync, Ok(0))
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fdatasync(fd: c_int) -> c_int {
    debug!("sys_fdatasync <= fd: {}", fd);
    syscall_body!(sys_fdatasync, Ok(0))
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_stat(path: *const c_char, buf: *mut core::ffi::c_void) -> c_int {
    let path = char_ptr_to_absolute_path(path);
    debug!("sys_stat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_stat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = ruxfs::fops::File::open(&path?, &options)?;
        let st: ctypes::stat = File::new(file).stat()?.into();

        #[cfg(not(feature = "musl"))]
        {
            let buf = buf as *mut ctypes::stat;
            unsafe { *buf = st };
            Ok(0)
        }

        #[cfg(feature = "musl")]
        {
            let kst = buf as *mut ctypes::kstat;
            unsafe {
                (*kst).st_dev = st.st_dev;
                (*kst).st_ino = st.st_ino;
                (*kst).st_mode = st.st_mode;
                (*kst).st_nlink = st.st_nlink;
                (*kst).st_uid = st.st_uid;
                (*kst).st_gid = st.st_gid;
                (*kst).st_size = st.st_size;
                (*kst).st_blocks = st.st_blocks;
                (*kst).st_blksize = st.st_blksize;
            }
            Ok(0)
        }
    })
}

/// retrieve information about the file pointed by `fd`
pub fn sys_fstat(fd: c_int, kst: *mut core::ffi::c_void) -> c_int {
    debug!("sys_fstat <= {} {:#x}", fd, kst as usize);
    syscall_body!(sys_fstat, {
        if kst.is_null() {
            return Err(LinuxError::EFAULT);
        }
        #[cfg(not(feature = "musl"))]
        {
            let buf = kst as *mut ctypes::stat;
            unsafe { *buf = get_file_like(fd)?.stat()?.into() };
            Ok(0)
        }
        #[cfg(feature = "musl")]
        {
            let st = get_file_like(fd)?.stat()?;
            let kst = kst as *mut ctypes::kstat;
            unsafe {
                (*kst).st_dev = st.st_dev;
                (*kst).st_ino = st.st_ino;
                (*kst).st_mode = st.st_mode;
                (*kst).st_nlink = st.st_nlink;
                (*kst).st_uid = st.st_uid;
                (*kst).st_gid = st.st_gid;
                (*kst).st_size = st.st_size;
                (*kst).st_blocks = st.st_blocks;
                (*kst).st_blksize = st.st_blksize;
                (*kst).st_atime_sec = st.st_atime.tv_sec;
                (*kst).st_atime_nsec = st.st_atime.tv_nsec;
                (*kst).st_mtime_sec = st.st_mtime.tv_sec;
                (*kst).st_mtime_nsec = st.st_mtime.tv_nsec;
                (*kst).st_ctime_sec = st.st_ctime.tv_sec;
                (*kst).st_ctime_nsec = st.st_ctime.tv_nsec;
                (*kst).st_rdev = st.st_rdev;
            }
            Ok(0)
        }
    })
}

/// Get the metadata of the symbolic link and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_lstat(path: *const c_char, buf: *mut ctypes::stat) -> ctypes::ssize_t {
    let path = char_ptr_to_absolute_path(path);
    debug!("sys_lstat <= {:?} {:#x}", path, buf as usize);
    syscall_body!(sys_lstat, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        unsafe { *buf = Default::default() }; // TODO
        Ok(0)
    })
}

/// `newfstatat` used by A64
pub unsafe fn sys_newfstatat(
    _fd: c_int,
    path: *const c_char,
    kst: *mut ctypes::kstat,
    flag: c_int,
) -> c_int {
    let path = char_ptr_to_absolute_path(path);
    debug!(
        "sys_newfstatat <= fd: {}, path: {:?}, flag: {:x}",
        _fd, path, flag
    );
    syscall_body!(sys_newfstatat, {
        if kst.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut options = OpenOptions::new();
        options.read(true);
        let file = ruxfs::fops::File::open(&path?, &options)?;
        let st = File::new(file).stat()?;
        unsafe {
            (*kst).st_dev = st.st_dev;
            (*kst).st_ino = st.st_ino;
            (*kst).st_mode = st.st_mode;
            (*kst).st_nlink = st.st_nlink;
            (*kst).st_uid = st.st_uid;
            (*kst).st_gid = st.st_gid;
            (*kst).st_size = st.st_size;
            (*kst).st_blocks = st.st_blocks;
            (*kst).st_blksize = st.st_blksize;
        }
        Ok(0)
    })
}

/// Get the path of the current directory.
pub fn sys_getcwd(buf: *mut c_char, size: usize) -> c_int {
    debug!("sys_getcwd <= {:#x} {}", buf as usize, size);
    syscall_body!(sys_getcwd, {
        if buf.is_null() {
            return Err(LinuxError::EINVAL);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, size as _) };
        let cwd = ruxfs::api::current_dir()?;
        let cwd = cwd.as_bytes();
        if cwd.len() < size {
            dst[..cwd.len()].copy_from_slice(cwd);
            dst[cwd.len()] = 0;
            Ok(cwd.len() + 1)
        } else {
            Err(LinuxError::ERANGE)
        }
    })
}

/// Rename `old` to `new`
/// If new exists, it is first removed.
///
/// Return 0 if the operation succeeds, otherwise return -1.
pub fn sys_rename(old: *const c_char, new: *const c_char) -> c_int {
    syscall_body!(sys_rename, {
        let old_path = char_ptr_to_absolute_path(old)?;
        let new_path = char_ptr_to_absolute_path(new)?;
        debug!("sys_rename <= old: {:?}, new: {:?}", old_path, new_path);
        ruxfs::api::rename(&old_path, &new_path)?;
        Ok(0)
    })
}

/// Rename at certain directory pointed by `oldfd`
///
/// TODO: only support `oldfd`, `newfd` equals to AT_FDCWD
pub fn sys_renameat(oldfd: c_int, old: *const c_char, newfd: c_int, new: *const c_char) -> c_int {
    let old_path = char_ptr_to_absolute_path(old);
    let new_path = char_ptr_to_absolute_path(new);
    debug!(
        "sys_renameat <= oldfd: {}, old: {:?}, newfd: {}, new: {:?}",
        oldfd, old_path, newfd, new_path
    );
    assert_eq!(oldfd, ctypes::AT_FDCWD as c_int);
    assert_eq!(newfd, ctypes::AT_FDCWD as c_int);
    syscall_body!(sys_renameat, {
        ruxfs::api::rename(&old_path?, &new_path?)?;
        Ok(0)
    })
}

/// Remove a directory, which must be empty
pub fn sys_rmdir(pathname: *const c_char) -> c_int {
    syscall_body!(sys_rmdir, {
        let path = char_ptr_to_absolute_path(pathname)?;
        debug!("sys_rmdir <= path: {:?}", path);
        ruxfs::api::remove_dir(&path)?;
        Ok(0)
    })
}

/// Removes a file from the filesystem.
pub fn sys_unlink(pathname: *const c_char) -> c_int {
    syscall_body!(sys_unlink, {
        let path = char_ptr_to_absolute_path(pathname)?;
        debug!("sys_unlink <= path: {:?}", path);
        ruxfs::api::remove_file(&path)?;
        Ok(0)
    })
}

/// deletes a name from the filesystem
pub fn sys_unlinkat(fd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    debug!(
        "sys_unlinkat <= fd: {}, pathname: {:?}, flags: {}",
        fd,
        char_ptr_to_absolute_path(pathname),
        flags
    );
    if flags as u32 & ctypes::AT_REMOVEDIR != 0 {
        return sys_rmdir(pathname);
    }
    sys_unlink(pathname)
}

/// Creates a new, empty directory at the provided path.
pub fn sys_mkdir(pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    // TODO: implement mode
    syscall_body!(sys_mkdir, {
        let path = char_ptr_to_absolute_path(pathname)?;
        debug!("sys_mkdir <= path: {:?}, mode: {:?}", path, mode);
        ruxfs::api::create_dir(&path)?;
        Ok(0)
    })
}

/// attempts to create a directory named pathname under directory pointed by `fd`
///
/// TODO: currently fd is not used
pub fn sys_mkdirat(fd: c_int, pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    debug!(
        "sys_mkdirat <= fd: {}, pathname: {:?}, mode: {:x?}",
        fd,
        char_ptr_to_absolute_path(pathname),
        mode
    );
    sys_mkdir(pathname, mode)
}

/// Changes the ownership of the file referred to by the open file descriptor fd
pub fn sys_fchownat(
    fd: c_int,
    path: *const c_char,
    uid: ctypes::uid_t,
    gid: ctypes::gid_t,
    flag: c_int,
) -> c_int {
    debug!(
        "sys_fchownat <= fd: {}, path: {:?}, uid: {}, gid: {}, flag: {}",
        fd,
        char_ptr_to_absolute_path(path),
        uid,
        gid,
        flag
    );
    syscall_body!(sys_fchownat, Ok(0))
}

/// read value of a symbolic link relative to directory file descriptor
/// TODO: currently only support symlink, so return EINVAL anyway
pub fn sys_readlinkat(
    fd: c_int,
    pathname: *const c_char,
    buf: *mut c_char,
    bufsize: usize,
) -> usize {
    let path = char_ptr_to_absolute_path(pathname);
    debug!(
        "sys_readlinkat <= path = {:?}, fd = {:}, buf = {:p}, bufsize = {:}",
        path, fd, buf, bufsize
    );
    syscall_body!(sys_readlinkat, {
        Err::<usize, LinuxError>(LinuxError::EINVAL)
    })
}

type LinuxDirent64 = ctypes::dirent;

fn convert_name_to_array(name: &[u8]) -> [i8; 256] {
    let mut array = [0i8; 256];
    let len = name.len();
    let name_ptr = name.as_ptr() as *const i8;
    let array_ptr = array.as_mut_ptr();

    unsafe {
        core::ptr::copy_nonoverlapping(name_ptr, array_ptr, len);
    }

    array
}

/// Read directory entries from a directory file descriptor.
///
/// TODO: check errors, change 280 to a special value
pub unsafe fn sys_getdents64(
    fd: c_int,
    dirent: *mut LinuxDirent64,
    count: ctypes::size_t,
) -> c_long {
    debug!(
        "sys_getdents64 <= fd: {}, dirent: {:p}, count: {}",
        fd, dirent, count
    );

    syscall_body!(sys_getdents64, {
        let expect_entries = count / 280;
        let dir = Directory::from_fd(fd)?;
        let mut my_dirent: Vec<DirEntry> =
            (0..expect_entries).map(|_| DirEntry::default()).collect();

        let n = dir.inner.lock().read_dir(&mut my_dirent)?;

        for (i, entry) in my_dirent.iter().enumerate() {
            let linux_dirent = LinuxDirent64 {
                d_ino: 1,
                d_off: 280,
                d_reclen: 280,
                d_type: entry.entry_type() as u8,
                d_name: convert_name_to_array(entry.name_as_bytes()),
            };

            unsafe {
                core::ptr::write(dirent.add(i), linux_dirent);
            }
        }

        Ok(n * 280)
    })
}

/// Reads `iocnt` buffers from the file associated with the file descriptor `fd` into the
/// buffers described by `iov`, starting at the position given by `offset`
pub unsafe fn sys_preadv(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_preadv <= fd: {}, iocnt: {}, offset: {}",
        fd, iocnt, offset
    );
    syscall_body!(sys_preadv, {
        if !(0..=1024).contains(&iocnt) {
            return Err(LinuxError::EINVAL);
        }

        let iovs = unsafe { core::slice::from_raw_parts(iov, iocnt as usize) };
        let mut ret = 0;
        for iov in iovs.iter() {
            if iov.iov_base.is_null() {
                continue;
            }
            ret += sys_pread64(fd, iov.iov_base, iov.iov_len, offset);
        }
        Ok(ret)
    })
}

/// checks accessibility to the file `pathname`.
/// If pathname is a symbolic link, it is dereferenced.
/// The mode is either the value F_OK, for the existence of the file,
/// or a mask consisting of the bitwise OR of one or more of R_OK, W_OK, and X_OK, for the read, write, execute permissions.
pub fn sys_faccessat(dirfd: c_int, pathname: *const c_char, mode: c_int, flags: c_int) -> c_int {
    let path = char_ptr_to_absolute_path(pathname).unwrap();
    debug!(
        "sys_faccessat <= dirfd {} path {} mode {} flags {}",
        dirfd, path, mode, flags
    );
    syscall_body!(sys_faccessat, {
        let mut options = OpenOptions::new();
        options.read(true);
        let _file = ruxfs::fops::File::open(&path, &options)?;
        Ok(0)
    })
}

/// changes the current working directory to the directory specified in path.
pub fn sys_chdir(path: *const c_char) -> c_int {
    let p = char_ptr_to_absolute_path(path).unwrap();
    debug!("sys_chdir <= path: {}", p);
    syscall_body!(sys_chdir, {
        set_current_dir(&p)?;
        Ok(0)
    })
}

/// from char_ptr get absolute_path_str
pub fn char_ptr_to_absolute_path<'a>(ptr: *const c_char) -> LinuxResult<Cow<'a, str>> {
    if ptr.is_null() {
        return Err(LinuxError::EFAULT);
    }

    let path = unsafe {
        let cstr = CStr::from_ptr(ptr);
        cstr.to_str().map_err(|_| LinuxError::EINVAL)?
    };

    if path.starts_with("..") {
        let stripped = path.strip_prefix("..").unwrap();
        let mut cwd = ruxfs::api::current_dir()?;
        if let Some(index) = cwd.rfind('/') {
            cwd.truncate(index);
            if let Some(index) = cwd.rfind('/') {
                cwd.truncate(index);
            }
        }
        let absolute_path: String = cwd + stripped;
        Ok(Cow::Owned(absolute_path))
    } else if path.starts_with('.') {
        let stripped = path.strip_prefix('.').unwrap();
        let cwd = ruxfs::api::current_dir()?;
        let absolute_path: String = cwd + stripped;
        Ok(Cow::Owned(absolute_path))
    } else {
        Ok(Cow::Borrowed(path))
    }
}
