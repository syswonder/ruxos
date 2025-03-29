/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;

use axerrno::LinuxError;
use ruxfdtable::{OpenFlags, RuxStat, RuxTimeSpec, RUX_FILE_LIMIT};
use ruxtask::current;
pub use ruxtask::fs::{close_file_like, get_file_like};

use crate::ctypes;

impl From<ctypes::timespec> for RuxTimeSpec {
    fn from(ctimespec: ctypes::timespec) -> Self {
        RuxTimeSpec {
            tv_sec: ctimespec.tv_sec,
            tv_nsec: ctimespec.tv_nsec,
        }
    }
}

impl From<ctypes::stat> for RuxStat {
    #[cfg(target_arch = "aarch64")]
    fn from(cstat: ctypes::stat) -> Self {
        RuxStat {
            st_dev: cstat.st_dev,
            st_ino: cstat.st_ino,
            st_mode: cstat.st_mode,
            st_nlink: cstat.st_nlink,
            st_uid: cstat.st_uid,
            st_gid: cstat.st_gid,
            st_rdev: cstat.st_rdev,
            __pad: cstat.__pad,
            st_size: cstat.st_size,
            st_blksize: cstat.st_blksize,
            __pad2: cstat.__pad2,
            st_blocks: cstat.st_blocks,
            st_atime: RuxTimeSpec::from(cstat.st_atime),
            st_mtime: RuxTimeSpec::from(cstat.st_mtime),
            st_ctime: RuxTimeSpec::from(cstat.st_ctime),
            __unused: cstat.__unused,
        }
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "riscv64"))]
    fn from(cstat: ctypes::stat) -> Self {
        RuxStat {
            st_dev: cstat.st_dev,
            st_ino: cstat.st_ino,
            st_nlink: cstat.st_nlink,
            st_mode: cstat.st_mode,
            st_uid: cstat.st_uid,
            st_gid: cstat.st_gid,
            __pad0: cstat.__pad0,
            st_rdev: cstat.st_rdev,
            st_size: cstat.st_size,
            st_blksize: cstat.st_blksize,
            st_blocks: cstat.st_blocks,
            st_atime: RuxTimeSpec::from(cstat.st_atime),
            st_mtime: RuxTimeSpec::from(cstat.st_mtime),
            st_ctime: RuxTimeSpec::from(cstat.st_ctime),
            __unused: cstat.__unused,
        }
    }
}

impl From<RuxTimeSpec> for ctypes::timespec {
    fn from(rtimespec: RuxTimeSpec) -> Self {
        ctypes::timespec {
            tv_sec: rtimespec.tv_sec,
            tv_nsec: rtimespec.tv_nsec,
        }
    }
}

impl From<RuxStat> for ctypes::stat {
    #[cfg(target_arch = "aarch64")]
    fn from(rstat: RuxStat) -> Self {
        ctypes::stat {
            st_dev: rstat.st_dev,
            st_ino: rstat.st_ino,
            st_mode: rstat.st_mode,
            st_nlink: rstat.st_nlink,
            st_uid: rstat.st_uid,
            st_gid: rstat.st_gid,
            st_rdev: rstat.st_rdev,
            __pad: rstat.__pad,
            st_size: rstat.st_size,
            st_blksize: rstat.st_blksize,
            __pad2: rstat.__pad2,
            st_blocks: rstat.st_blocks,
            st_atime: rstat.st_atime.into(),
            st_mtime: rstat.st_mtime.into(),
            st_ctime: rstat.st_ctime.into(),
            __unused: rstat.__unused,
        }
    }

    #[cfg(any(target_arch = "x86_64", target_arch = "riscv64"))]
    fn from(rstat: RuxStat) -> Self {
        ctypes::stat {
            st_dev: rstat.st_dev,
            st_ino: rstat.st_ino,
            st_nlink: rstat.st_nlink,
            st_mode: rstat.st_mode,
            st_uid: rstat.st_uid,
            st_gid: rstat.st_gid,
            __pad0: rstat.__pad0,
            st_rdev: rstat.st_rdev,
            st_size: rstat.st_size,
            st_blksize: rstat.st_blksize,
            st_blocks: rstat.st_blocks,
            st_atime: rstat.st_atime.into(),
            st_mtime: rstat.st_mtime.into(),
            st_ctime: rstat.st_ctime.into(),
            __unused: rstat.__unused,
        }
    }
}

/// Close a file by `fd`.
pub fn sys_close(fd: c_int) -> c_int {
    debug!("sys_close <= {}", fd);
    if (0..=2).contains(&fd) {
        return 0; // stdin, stdout, stderr
    }
    syscall_body!(sys_close, close_file_like(fd).map(|_| 0))
}

/// Duplicate a file descriptor.
pub fn sys_dup(old_fd: c_int) -> c_int {
    debug!("sys_dup <= {}", old_fd);
    syscall_body!(sys_dup, {
        let binding_task = current();
        let mut binding_fs = binding_task.fs.lock();
        let new_fd = binding_fs.as_mut().unwrap().fd_table.dup(old_fd as _)?;
        Ok(new_fd as c_int)
    })
}

/// Duplicate a file descriptor, but it uses the file descriptor number specified in `new_fd`.
///
/// The close-on-exec flag for the duplicate descriptor is off.
pub fn sys_dup2(old_fd: c_int, new_fd: c_int) -> c_int {
    debug!("sys_dup2 <= old_fd: {}, new_fd: {}", old_fd, new_fd);
    syscall_body!(sys_dup2, {
        if old_fd == new_fd {
            // check if `oldfd` isn't an open file descriptor. If it not, return `EBADF`
            get_file_like(old_fd as _)?;
            return Ok(new_fd);
        }
        if new_fd as usize >= RUX_FILE_LIMIT {
            return Err(LinuxError::EBADF);
        }

        // The steps of closing and reusing the file descriptor newfd are performed atomically.
        current()
            .fs
            .lock()
            .as_mut()
            .unwrap()
            .fd_table
            .dup3(old_fd as _, new_fd as _, false)?;

        Ok(new_fd)
    })
}

/// `dup3` used by A64 for MUSL
#[cfg(feature = "musl")]
pub fn sys_dup3(old_fd: c_int, new_fd: c_int, flags: c_int) -> c_int {
    debug!(
        "sys_dup3 <= old_fd: {}, new_fd: {}, flags: {:x}",
        old_fd, new_fd, flags
    );
    syscall_body!(sys_dup3, {
        if old_fd == new_fd {
            return Err(LinuxError::EINVAL);
        }
        let cloexec = (flags as u32 & ctypes::O_CLOEXEC) != 0;
        let binding_task = current();
        let mut binding_fs = binding_task.fs.lock();
        binding_fs
            .as_mut()
            .unwrap()
            .fd_table
            .dup3(old_fd as _, new_fd as _, cloexec)?;

        Ok(new_fd)
    })
}

/// Manipulate file descriptor.
///
/// TODO: `SET/GET` command is ignored, hard-code stdin/stdout
pub fn sys_fcntl(fd: c_int, cmd: c_int, arg: usize) -> c_int {
    debug!("sys_fcntl <= fd: {} cmd: {} arg: {}", fd, cmd, arg);
    syscall_body!(sys_fcntl, {
        match cmd as u32 {
            ctypes::F_DUPFD => {
                let new_fd = current()
                    .fs
                    .lock()
                    .as_mut()
                    .unwrap()
                    .fd_table
                    .dup_with_low_bound(fd as _, arg as _, false)?;
                Ok(new_fd as _)
            }
            ctypes::F_GETFD => {
                let binding_task = current();
                let mut binding_fs = binding_task.fs.lock();
                let fd_table = &mut binding_fs.as_mut().unwrap().fd_table;
                if fd_table.get(fd as _).is_none() {
                    return Err(LinuxError::EBADF);
                }
                let cloexec = if fd_table.get_cloexec(fd as _) {
                    ctypes::FD_CLOEXEC
                } else {
                    0
                };
                Ok(cloexec as _)
            }
            ctypes::F_DUPFD_CLOEXEC => {
                let new_fd = current()
                    .fs
                    .lock()
                    .as_mut()
                    .unwrap()
                    .fd_table
                    .dup_with_low_bound(fd as _, arg as _, true)?;
                Ok(new_fd as _)
            }
            ctypes::F_SETFL => {
                // Set the file status flags to the value specified by `arg`
                let f = get_file_like(fd)?;
                let old_access_flags = f.flags() & OpenFlags::O_ACCMODE;
                f.set_flags(
                    old_access_flags | OpenFlags::from_bits_truncate(arg as _).status_flags(),
                )?;
                Ok(0)
            }
            ctypes::F_GETFL => {
                // Return the file access mode and the file status flags; `arg` is ignored.
                Ok(get_file_like(fd)?.flags().getfl().bits())
            }
            ctypes::F_SETFD => {
                let cloexec =
                    OpenFlags::from_bits_truncate(arg as _).contains(OpenFlags::O_CLOEXEC);
                let binding_task = current();
                let mut binding_fs = binding_task.fs.lock();
                let fd_table = &mut binding_fs.as_mut().unwrap().fd_table;
                if fd_table.get(fd as _).is_none() {
                    return Err(LinuxError::EBADF);
                }
                fd_table.set_cloexec(fd as _, cloexec);
                Ok(0)
            }
            _ => {
                warn!("unsupported fcntl parameters: cmd {}", cmd);
                Ok(0)
            }
        }
    })
}
