/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::string::String;
use alloc::sync::Arc;
use core::{
    ffi::{c_char, c_int, c_long, c_ulong, c_void, CStr},
    str,
};

use ruxfs::{api::FileType, fops::lookup, FilePerm, MountPoint};

use axerrno::{LinuxError, LinuxResult};
use axio::{Error, SeekFrom};
use ruxfdtable::{OpenFlags, RuxStat};
use ruxfs::{
    fops::{self, open_file_like},
    AbsPath, DirEntry, Directory, File, RelPath,
};

use crate::{ctypes, utils::char_ptr_to_str};
use ruxtask::fs::{add_file_like, get_file_like, get_umask};

use super::stdio::{Stdin, Stdout};

struct InitFsImpl;

#[crate_interface::impl_interface]
impl ruxtask::fs::InitFs for InitFsImpl {
    fn add_stdios_to_fd_table(fs: &mut ruxtask::fs::FileSystem) {
        debug!("init initial process's fd_table");
        let fd_table = &mut fs.fd_table;
        fd_table.add(Arc::new(Stdin::default()) as _, OpenFlags::empty()); // stdin
        fd_table.add(Arc::new(Stdout {}) as _, OpenFlags::empty()); // stdout
        fd_table.add(Arc::new(Stdout {}) as _, OpenFlags::empty()); // stderr
    }
}

/// Open a file by `filename` and insert it into the file descriptor table.
///
/// Return its index in the file table (`fd`). Return `EMFILE` if it already
/// has the maximum number of files open.
pub fn sys_open(filename: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    sys_openat(ctypes::AT_FDCWD, filename, flags, mode)
}

/// Open a file under a specific dir
pub fn sys_openat(fd: c_int, path: *const c_char, flags: c_int, mode: ctypes::mode_t) -> c_int {
    syscall_body!(sys_openat, {
        let path = parse_path_at(fd, path)?;
        let flags = OpenFlags::from_bits(flags).ok_or(LinuxError::EINVAL)?;
        debug!("sys_openat <= fd {fd} {path:?}, {flags:?}, {mode:#o}");
        let mode = FilePerm::from_bits_truncate(mode as u16 & !get_umask());
        add_file_like(open_file_like(&path, flags, mode)?, flags)
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
    syscall_body!(sys_pread64, {
        debug!("sys_pread64 <= {fd} {count} {pos}");
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };

        let size = file_from_fd(fd)?.read_at(pos as u64, dst)?;
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
    syscall_body!(sys_pwrite64, {
        debug!("sys_pwrite64 <= {fd} {count} {pos}");
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let src = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
        let size = file_from_fd(fd)?.write_at(pos as u64, src)?;
        Ok(size as ctypes::ssize_t)
    })
}

/// Set the position of the file indicated by `fd`.
///
/// Return its position after seek.
pub fn sys_lseek(fd: c_int, offset: ctypes::off_t, whence: c_int) -> ctypes::off_t {
    syscall_body!(sys_lseek, {
        debug!("sys_lseek <= {fd} {offset} {whence}");
        let pos = match whence {
            0 => SeekFrom::Start(offset as _),
            1 => SeekFrom::Current(offset as _),
            2 => SeekFrom::End(offset as _),
            _ => return Err(LinuxError::EINVAL),
        };
        let off = file_from_fd(fd)?.seek(pos)?;
        Ok(off)
    })
}

/// Truncate a file to a specified length.
pub unsafe fn sys_ftruncate(fd: c_int, length: ctypes::off_t) -> c_int {
    syscall_body!(sys_ftruncate, {
        debug!("sys_ftruncate <= {fd} {length}");
        let file = file_from_fd(fd)?;
        file.truncate(length as u64)?;
        Ok(0)
    })
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fsync(fd: c_int) -> c_int {
    debug!("sys_fsync <= fd: {fd}");
    syscall_body!(sys_fsync, Ok(0))
}

/// Synchronize a file's in-core state with storage device
///
/// TODO
pub unsafe fn sys_fdatasync(fd: c_int) -> c_int {
    debug!("sys_fdatasync <= fd: {fd}");
    syscall_body!(sys_fdatasync, Ok(0))
}

/// Get the file metadata by `path` and write into `buf`.
///
/// Return 0 if success.
pub unsafe fn sys_stat(path: *const c_char, buf: *mut core::ffi::c_void) -> c_int {
    syscall_body!(sys_stat, {
        let path = parse_path(path)?;
        debug!("sys_stat <= {:?} {:#x}", path, buf as usize);
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let node = fops::lookup(&path)?;
        let st = RuxStat::from(node.get_attr()?);

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
    syscall_body!(sys_fstat, {
        debug!("sys_fstat <= {} {:#x}", fd, kst as usize);
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
    syscall_body!(sys_lstat, {
        let path = parse_path(path)?;
        debug!("sys_lstat <= {:?} {:#x}", path, buf as usize);
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        unsafe { *buf = Default::default() }; // TODO
        Ok(0)
    })
}

/// `newfstatat` used by A64
pub unsafe fn sys_newfstatat(
    fd: c_int,
    path: *const c_char,
    kst: *mut ctypes::kstat,
    flag: c_int,
) -> c_int {
    syscall_body!(sys_newfstatat, {
        let path = parse_path_at(fd, path)?;
        debug!("sys_newfstatat <= fd: {fd}, path: {path:?}, flag: {flag:x}");
        if kst.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let node = fops::lookup(&path)?;
        let st = RuxStat::from(node.get_attr()?);

        // TODO: remove this initialization when fields are fully implemented
        unsafe {
            core::ptr::write_bytes(kst, 0, 1);
        }

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
        let dst: &mut [u8] = unsafe { core::slice::from_raw_parts_mut(buf as _, size as _) };
        let cwd = fops::current_dir()?;
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
        let old = parse_path(old)?;
        let new = parse_path(new)?;
        debug!("sys_rename <= old: {old:?}, new: {new:?}");
        if old == new {
            return Ok(0);
        }
        match fops::lookup(&old) {
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
        match fops::lookup(&new) {
            Ok(_) => return Err(LinuxError::EEXIST),
            Err(Error::NotFound) => {}
            Err(e) => return Err(e.into()),
        }
        fops::rename(&old, &new)?;
        Ok(0)
    })
}

/// Rename at certain directory pointed by `oldfd`
pub fn sys_renameat(oldfd: c_int, old: *const c_char, newfd: c_int, new: *const c_char) -> c_int {
    syscall_body!(sys_renameat, {
        let old_path = parse_path_at(oldfd, old)?;
        let new_path = parse_path_at(newfd, new)?;
        debug!(
            "sys_renameat <= oldfd: {oldfd}, old: {old_path:?}, newfd: {newfd}, new: {new_path:?}"
        );
        fops::rename(&old_path, &new_path)?;
        Ok(0)
    })
}

/// Remove a directory, which must be empty
pub fn sys_rmdir(pathname: *const c_char) -> c_int {
    syscall_body!(sys_rmdir, {
        let path = parse_path(pathname)?;
        debug!("sys_rmdir <= path: {path:?}");
        match fops::lookup(&path) {
            Ok(node) => {
                let attr = node.get_attr()?;
                if !attr.is_dir() {
                    return Err(LinuxError::ENOTDIR);
                }
                if fops::is_mount_point(&path) {
                    return Err(LinuxError::EPERM);
                }
                if !attr.perm().owner_writable() {
                    return Err(LinuxError::EPERM);
                }
                if !node.is_empty()? {
                    return Err(LinuxError::ENOTEMPTY);
                }
                fops::remove_dir(&path)?;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(0)
    })
}

/// Removes a file from the filesystem.
pub fn sys_unlink(pathname: *const c_char) -> c_int {
    syscall_body!(sys_unlink, {
        let path = parse_path(pathname)?;
        debug!("sys_unlink <= path: {path:?}");
        match fops::lookup(&path) {
            Ok(node) => {
                let attr = node.get_attr()?;
                if attr.is_dir() {
                    return Err(LinuxError::EISDIR);
                }
                if !attr.perm().owner_writable() {
                    return Err(LinuxError::EPERM);
                }
                fops::remove_file(&path)?;
            }
            Err(e) => return Err(e.into()),
        }
        Ok(0)
    })
}

/// deletes a name from the filesystem
pub fn sys_unlinkat(fd: c_int, pathname: *const c_char, flags: c_int) -> c_int {
    syscall_body!(sys_unlinkat, {
        let path = parse_path_at(fd, pathname)?;
        let rmdir = flags as u32 & ctypes::AT_REMOVEDIR != 0;
        debug!("sys_unlinkat <= fd: {fd}, pathname: {path:?}, flags: {flags}");
        match fops::lookup(&path) {
            Ok(node) => {
                let attr = node.get_attr()?;
                if rmdir {
                    if !attr.is_dir() {
                        return Err(LinuxError::ENOTDIR);
                    }
                    if fops::is_mount_point(&path) {
                        return Err(LinuxError::EPERM);
                    }
                    if !attr.perm().owner_writable() {
                        return Err(LinuxError::EPERM);
                    }
                    if !node.is_empty()? {
                        return Err(LinuxError::ENOTEMPTY);
                    }
                    fops::remove_dir(&path)?;
                } else {
                    if attr.is_dir() {
                        return Err(LinuxError::EISDIR);
                    }
                    if !attr.perm().owner_writable() {
                        return Err(LinuxError::EPERM);
                    }
                    fops::remove_file(&path)?;
                }
            }
            Err(e) => return Err(e.into()),
        }
        Ok(0)
    })
}

/// Change permissions of a file
pub fn sys_fchmodat(dirfd: c_int, pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    syscall_body!(sys_mknodat, {
        let path = parse_path_at(dirfd, pathname)?;
        debug!("sys_fchmodat <= fd: {dirfd}, path: {path:?}, mode: {mode:#o}");
        let node = lookup(&path)?;
        node.set_mode(FilePerm::from_bits_truncate(mode as u16))?;
        Ok(0)
    })
}

/// Creates a new, empty file at the provided path.
pub fn sys_mknodat(
    fd: c_int,
    pathname: *const c_char,
    mode: ctypes::mode_t,
    _dev: ctypes::dev_t,
) -> c_int {
    // TODO: implement permissions mode
    syscall_body!(sys_mknodat, {
        let path = parse_path_at(fd, pathname)?;
        debug!("sys_mknodat <= fd: {fd}, pathname: {path:?}, mode: {mode:#o}, dev: {_dev:x?}");
        let file_type = match mode & ctypes::S_IFMT {
            ctypes::S_IFREG => FileType::File,
            ctypes::S_IFIFO => FileType::Fifo,
            _ => todo!(),
        };
        let mode = FilePerm::from_bits_truncate(mode as u16 & !get_umask());
        fops::create(&path, file_type, mode)?;
        Ok(0)
    })
}

/// Creates a new, empty directory at the provided path.
pub fn sys_mkdir(pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    sys_mkdirat(ctypes::AT_FDCWD, pathname, mode)
}

/// attempts to create a directory named pathname under directory pointed by `fd`
pub fn sys_mkdirat(fd: c_int, pathname: *const c_char, mode: ctypes::mode_t) -> c_int {
    syscall_body!(sys_mkdirat, {
        let path = parse_path_at(fd, pathname)?;
        debug!("sys_mkdirat <= fd: {fd}, pathname: {path:?}, mode: {mode:#o}",);
        let mode = FilePerm::from_bits_truncate(mode as u16 & !get_umask());
        match fops::lookup(&path) {
            Ok(_) => return Err(LinuxError::EEXIST),
            Err(Error::NotFound) => fops::create(&path, FileType::Dir, mode)?,
            Err(e) => return Err(e.into()),
        }
        Ok(0)
    })
}

/// Changes the ownership of the file referred to by the open file descriptor fd
pub fn sys_fchownat(
    fd: c_int,
    path: *const c_char,
    uid: ctypes::uid_t,
    gid: ctypes::gid_t,
    flag: c_int,
) -> c_int {
    syscall_body!(sys_fchownat, {
        let path = parse_path_at(fd, path)?;
        debug!("sys_fchownat <= fd: {fd}, path: {path:?}, uid: {uid}, gid: {gid}, flag: {flag}");
        Ok(0)
    })
}

/// read value of a symbolic link relative to directory file descriptor
/// TODO: currently only support symlink, so return EINVAL anyway
pub fn sys_readlinkat(
    fd: c_int,
    pathname: *const c_char,
    buf: *mut c_char,
    bufsize: usize,
) -> usize {
    syscall_body!(sys_readlinkat, {
        let path = parse_path_at(fd, pathname)?;
        debug!(
            "sys_readlinkat <= path = {path:?}, fd = {fd:}, buf = {buf:p}, bufsize = {bufsize:}"
        );
        Err::<usize, LinuxError>(LinuxError::EINVAL)
    })
}

type LinuxDirent64 = ctypes::dirent;
/// `d_ino` + `d_off` + `d_reclen` + `d_type`
const DIRENT64_FIXED_SIZE: usize = 19;

/// Read directory entries from a directory file descriptor.
pub unsafe fn sys_getdents64(fd: c_int, dirp: *mut LinuxDirent64, count: ctypes::size_t) -> c_long {
    debug!("sys_getdents64 <= fd: {fd}, dirp: {dirp:p}, count: {count}");
    syscall_body!(sys_getdents64, {
        if count < DIRENT64_FIXED_SIZE {
            return Err(LinuxError::EINVAL);
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(dirp as *mut u8, count) };
        // EBADFD handles here
        let dir = dir_from_fd(fd)?;
        // bytes written in buf
        let mut written = 0;

        loop {
            let mut entry = [DirEntry::default()];
            let offset = dir.entry_idx();
            let n = dir.read_dir(&mut entry)?;
            debug!(
                "entry {:?}",
                str::from_utf8(entry[0].name_as_bytes()).unwrap()
            );
            if n == 0 {
                return Ok(written as isize);
            }
            let entry = &entry[0];

            let name = entry.name_as_bytes();
            let name_len = name.len();
            let entry_size = (DIRENT64_FIXED_SIZE + name_len + 1 + 7) & !7; // align to 8 bytes

            // buf not big enough to hold the entry
            if written + entry_size > count {
                debug!("buf not big enough");
                // revert the offset
                dir.set_entry_idx(offset);
                break;
            }

            // write entry to buffer
            let dirent: &mut LinuxDirent64 =
                unsafe { &mut *(buf.as_mut_ptr().add(written) as *mut LinuxDirent64) };
            // set fixed-size fields
            dirent.d_ino = 1;
            dirent.d_off = (offset + 1) as i64;
            dirent.d_reclen = entry_size as u16;
            dirent.d_type = entry.entry_type() as u8;
            // set file name
            dirent.d_name[..name_len].copy_from_slice(unsafe {
                core::slice::from_raw_parts(name.as_ptr() as *const c_char, name_len)
            });
            dirent.d_name[name_len] = 0;

            written += entry_size;
        }

        Ok(written as isize)
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
    debug!("sys_preadv <= fd: {fd}, iocnt: {iocnt}, offset: {offset}");
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
    syscall_body!(sys_faccessat, {
        let path = parse_path_at(dirfd, pathname)?;
        debug!("sys_faccessat <= dirfd {dirfd} path {path} mode {mode} flags {flags}");
        Ok(0)
    })
}

/// changes the current working directory to the directory specified in path.
pub fn sys_chdir(path: *const c_char) -> c_int {
    syscall_body!(sys_chdir, {
        let path = parse_path(path)?;
        debug!("sys_chdir <= path: {path:?}");
        fops::set_current_dir(path)?;
        Ok(0)
    })
}

pub const MS_NODEV: u32 = 2;
pub const MS_NOSUID: u32 = 4;

/// umount a filesystem at a specific location in the filesystem tree
pub fn sys_umount2(target: *const c_char, flags: c_int) -> c_int {
    info!(
        "sys_umount2 <= target: {:?}, flags: {:#x}",
        char_ptr_to_str(target),
        flags
    );
    syscall_body!(sys_umount2, {
        let target = char_ptr_to_str(target)?;
        let dir = ruxtask::current()
            .fs
            .lock()
            .as_mut()
            .unwrap()
            .root_dir
            .clone();
        dir.umount(&AbsPath::new(target));
        Ok(0)
    })
}

/// mount a filesystem at a specific location in the filesystem tree
pub fn sys_mount(
    source: *const c_char,
    raw_target: *const c_char,
    filesystemtype: *const c_char,
    mountflags: c_ulong,
    data: *const c_void,
) -> c_int {
    info!(
        "sys_mount <= source: {:?}, target: {:?}, filesystemtype: {:?}, mountflags: {:#x}, data: {:p}",
        char_ptr_to_str(source),
        char_ptr_to_str(raw_target),
        char_ptr_to_str(filesystemtype),
        mountflags,
        data
    );
    syscall_body!(sys_mount, {
        let f1 = MS_NODEV; //ctypes::MS_NODEV;
        let f2 = MS_NOSUID; //ctypes::MS_NOSUID;
        info!(
            "mount flags: {:#x}, f1: {:#}, f2: {:#}, flag: {:#}",
            mountflags,
            f1,
            f2,
            f1 | f2
        );
        if mountflags != (f1 | f2).into() {
            warn!("mount flags not supported: {mountflags:#x}");
        }

        let target = char_ptr_to_str(raw_target)?;
        let target = String::from(target);
        let dir = ruxtask::current()
            .fs
            .lock()
            .as_mut()
            .unwrap()
            .root_dir
            .clone();
        let vfsops = ruxfuse::fuse::fusefs();
        info!("mounting filesystem at {target}");
        dir.mount(MountPoint {
            path: target,
            fs: vfsops,
        })?;
        Ok(0)
    })
}

/// perform a memory barrier operation.
pub fn sys_membarrier(cmd: c_int, flags: c_int) -> c_int {
    info!("sys_membarrier <= cmd: {cmd}, flags: {flags}");
    syscall_body!(sys_membarrier, Ok(0))
}

/// from char_ptr get path_str
fn char_ptr_to_path_str<'a>(ptr: *const c_char) -> LinuxResult<&'a str> {
    if ptr.is_null() {
        return Err(LinuxError::EFAULT);
    }
    unsafe {
        let cstr = CStr::from_ptr(ptr);
        cstr.to_str().map_err(|_| LinuxError::EINVAL)
    }
}

/// Parse `path` argument for fs syscalls.
///
/// * If the given `path` is absolute, return it as is.
/// * If the given `path` is relative, join it against the current working directory.
pub fn parse_path(path: *const c_char) -> LinuxResult<AbsPath<'static>> {
    let path = char_ptr_to_path_str(path)?;
    if path.starts_with('/') {
        Ok(AbsPath::new_canonicalized(path))
    } else {
        Ok(fops::current_dir()?.join(&RelPath::new_canonicalized(path)))
    }
}

/// Parse `path` and `dirfd` arguments for fs syscalls.
///
/// * If the given `path` is absolute, return it as is.
/// * If the given `path` is relative and `dirfd` is `AT_FDCWD`, join it against the
///   current working directory.
/// * If the given `path` is relative and `dirfd` is not `AT_FDCWD`, join it against the
///   directory of the file descriptor.
pub fn parse_path_at(dirfd: c_int, path: *const c_char) -> LinuxResult<AbsPath<'static>> {
    let path = char_ptr_to_path_str(path)?;
    if path.starts_with('/') {
        Ok(AbsPath::new_canonicalized(path))
    } else if dirfd == ctypes::AT_FDCWD {
        Ok(fops::current_dir()?.join(&RelPath::new_canonicalized(path)))
    } else {
        Ok(dir_from_fd(dirfd)?
            .path()
            .join(&RelPath::new_canonicalized(path)))
    }
}

fn file_from_fd(fd: i32) -> LinuxResult<Arc<File>> {
    get_file_like(fd)?
        .into_any()
        .downcast::<File>()
        .map_err(|_| LinuxError::EINVAL)
}

fn dir_from_fd(fd: i32) -> LinuxResult<Arc<Directory>> {
    get_file_like(fd)?
        .into_any()
        .downcast::<Directory>()
        .map_err(|_| LinuxError::EINVAL)
}
