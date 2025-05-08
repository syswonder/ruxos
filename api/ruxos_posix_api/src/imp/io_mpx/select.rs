/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use ruxhal::time::current_time;

use crate::ctypes;
use ruxtask::fs::get_file_like;

const FD_SETSIZE: usize = 1024;
const BITS_PER_USIZE: usize = usize::BITS as usize;
const FD_SETSIZE_USIZES: usize = FD_SETSIZE.div_ceil(BITS_PER_USIZE);

struct FdSets {
    nfds: usize,
    bits: [usize; FD_SETSIZE_USIZES * 3],
}

impl FdSets {
    fn from(
        nfds: usize,
        read_fds: *const ctypes::fd_set,
        write_fds: *const ctypes::fd_set,
        except_fds: *const ctypes::fd_set,
    ) -> Self {
        let nfds = nfds.min(FD_SETSIZE);
        let nfds_usizes = nfds.div_ceil(BITS_PER_USIZE);
        let mut bits = core::mem::MaybeUninit::<[usize; FD_SETSIZE_USIZES * 3]>::uninit();
        let bits_ptr = unsafe { core::mem::transmute(bits.as_mut_ptr()) };

        let copy_from_fd_set = |bits_ptr: *mut usize, fds: *const ctypes::fd_set| unsafe {
            let dst = core::slice::from_raw_parts_mut(bits_ptr, nfds_usizes);
            if fds.is_null() {
                dst.fill(0);
            } else {
                let fds_ptr = (*fds).fds_bits.as_ptr() as *const usize;
                let src = core::slice::from_raw_parts(fds_ptr, nfds_usizes);
                dst.copy_from_slice(src);
            }
        };

        let bits = unsafe {
            copy_from_fd_set(bits_ptr, read_fds);
            copy_from_fd_set(bits_ptr.add(FD_SETSIZE_USIZES), write_fds);
            copy_from_fd_set(bits_ptr.add(FD_SETSIZE_USIZES * 2), except_fds);
            bits.assume_init()
        };
        Self { nfds, bits }
    }

    fn poll_all(
        &self,
        res_read_fds: *mut ctypes::fd_set,
        res_write_fds: *mut ctypes::fd_set,
        res_except_fds: *mut ctypes::fd_set,
    ) -> LinuxResult<usize> {
        let mut read_bits_ptr = self.bits.as_ptr();
        let mut write_bits_ptr = unsafe { read_bits_ptr.add(FD_SETSIZE_USIZES) };
        let mut execpt_bits_ptr = unsafe { read_bits_ptr.add(FD_SETSIZE_USIZES * 2) };
        let mut i = 0;
        let mut res_num = 0;
        while i < self.nfds {
            let read_bits = unsafe { *read_bits_ptr };
            let write_bits = unsafe { *write_bits_ptr };
            let except_bits = unsafe { *execpt_bits_ptr };
            unsafe {
                read_bits_ptr = read_bits_ptr.add(1);
                write_bits_ptr = write_bits_ptr.add(1);
                execpt_bits_ptr = execpt_bits_ptr.add(1);
            }

            let all_bits = read_bits | write_bits | except_bits;
            if all_bits == 0 {
                i += BITS_PER_USIZE;
                continue;
            }
            let mut j = 0;
            while j < BITS_PER_USIZE && i + j < self.nfds {
                let bit = 1 << j;
                if all_bits & bit == 0 {
                    j += 1;
                    continue;
                }
                let fd = i + j;
                match get_file_like(fd as _)?.poll() {
                    Ok(state) => {
                        if state.readable && read_bits & bit != 0 {
                            unsafe { set_fd_set(res_read_fds, fd) };
                            res_num += 1;
                        }
                        if state.writable && write_bits & bit != 0 {
                            unsafe { set_fd_set(res_write_fds, fd) };
                            res_num += 1;
                        }
                        if state.pollhup {
                            unsafe { set_fd_set(res_except_fds, fd) };
                            res_num += 1;
                        }
                    }
                    Err(e) => {
                        debug!("    except: {} {:?}", fd, e);
                        if except_bits & bit != 0 {
                            unsafe { set_fd_set(res_except_fds, fd) };
                            res_num += 1;
                        }
                    }
                }
                j += 1;
            }
            i += BITS_PER_USIZE;
        }
        Ok(res_num)
    }
}

/// Monitor multiple file descriptors, waiting until one or more of the file descriptors become "ready" for some class of I/O operation
pub unsafe fn sys_select(
    nfds: c_int,
    readfds: *mut ctypes::fd_set,
    writefds: *mut ctypes::fd_set,
    exceptfds: *mut ctypes::fd_set,
    timeout: *mut ctypes::timeval,
) -> c_int {
    debug!(
        "sys_select <= {} {:#x} {:#x} {:#x}",
        nfds, readfds as usize, writefds as usize, exceptfds as usize
    );
    syscall_body!(sys_select, {
        if nfds < 0 {
            return Err(LinuxError::EINVAL);
        }
        let nfds = (nfds as usize).min(FD_SETSIZE);
        let deadline = unsafe { timeout.as_ref().map(|t| current_time() + (*t).into()) };
        let fd_sets = FdSets::from(nfds, readfds, writefds, exceptfds);

        unsafe {
            zero_fd_set(readfds, nfds);
            zero_fd_set(writefds, nfds);
            zero_fd_set(exceptfds, nfds);
        }

        loop {
            #[cfg(feature = "net")]
            ruxnet::poll_interfaces();
            let res = fd_sets.poll_all(readfds, writefds, exceptfds)?;
            if res > 0 {
                return Ok(res);
            }

            if deadline.map_or(false, |ddl| current_time() >= ddl) {
                debug!("    timeout!");
                return Ok(0);
            }
            crate::sys_sched_yield();
        }
    })
}

/// allows  a  program  to  monitor  multiple  file descriptors, waiting until one or more of the file descriptors become
/// "ready" for some class of I/O operation (e.g., input possible)
///
/// TODO: signal is ignored currently
pub unsafe fn sys_pselect6(
    nfds: c_int,
    readfds: *mut ctypes::fd_set,
    writefds: *mut ctypes::fd_set,
    exceptfds: *mut ctypes::fd_set,
    timeout: *mut ctypes::timeval,
    _sigs: *const c_void,
) -> c_int {
    sys_select(nfds, readfds, writefds, exceptfds, timeout)
}

unsafe fn zero_fd_set(fds: *mut ctypes::fd_set, nfds: usize) {
    if !fds.is_null() {
        let nfds_usizes = nfds.div_ceil(BITS_PER_USIZE);
        let dst = &mut (*fds).fds_bits[..nfds_usizes];
        dst.fill(0);
    }
}

unsafe fn set_fd_set(fds: *mut ctypes::fd_set, fd: usize) {
    if !fds.is_null() {
        (*fds).fds_bits[fd / BITS_PER_USIZE] |= 1 << (fd % BITS_PER_USIZE);
    }
}
