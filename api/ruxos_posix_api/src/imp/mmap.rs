/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use alloc::alloc::{alloc, dealloc};
use core::{
    alloc::Layout,
    ffi::{c_int, c_void},
};

use axerrno::LinuxError;

/// Creates a new mapping in the virtual address space of the call‐
/// ing process.
///
/// TODO: Only support `start` equals to NULL, ignore fd, prot, flags
pub fn sys_mmap(
    start: *mut c_void,
    len: ctypes::size_t,
    _prot: c_int,
    _flags: c_int,
    _fd: c_int,
    _off: ctypes::off_t,
) -> *mut c_void {
    debug!("sys_mmap <= start: {:p}, len: {}, fd: {}", start, len, _fd);
    syscall_body!(sys_mmap, {
        if !start.is_null() {
            debug!("Do not support explicitly specifying start addr");
            return Ok(core::ptr::null_mut());
        }
        let layout = Layout::from_size_align(len, 8).unwrap();
        unsafe {
            let ptr = alloc(layout).cast::<c_void>();
            (ptr as *mut u8).write_bytes(0, len);
            assert!(!ptr.is_null(), "sys_mmap failed");
            Ok(ptr)
        }
    })
}

/// Deletes the mappings for the specified address range
pub fn sys_munmap(start: *mut c_void, len: ctypes::size_t) -> c_int {
    debug!("sys_munmap <= start: {:p}, len: {}", start, len);
    syscall_body!(sys_munmap, {
        if start.is_null() {
            return Err(LinuxError::EINVAL);
        }
        let layout = Layout::from_size_align(len, 8).unwrap();
        unsafe { dealloc(start.cast(), layout) }
        Ok(0)
    })
}

/// Changes the access protections for the calling process's memory pages
/// containing any part of the address range in the interval [addr, addr+len-1].  
/// addr must be aligned to a page boundary.
///
/// TODO: implement this
pub fn sys_mprotect(addr: *mut c_void, len: ctypes::size_t, prot: c_int) -> c_int {
    debug!(
        "sys_mprotect <= addr: {:p}, len: {}, prot: {}, Currently IGNORED",
        addr, len, prot
    );
    syscall_body!(sys_mprotect, Ok(0))
}

/// Remap a virtual memory address
///
/// TODO: only support
pub fn sys_mremap(
    old_addr: *mut c_void,
    old_size: ctypes::size_t,
    new_size: ctypes::size_t,
    _flags: c_int,
    _new_addr: *mut c_void,
) -> *mut c_void {
    debug!(
        "sys_mremap <= old_addr: {:p}, old_size: {}, new_size: {}, flags: {}, new_addr: {:p}",
        old_addr, old_size, new_size, _flags, _new_addr
    );
    syscall_body!(sys_mremap, {
        if old_addr.is_null() {
            // TODO: It should be ctypes::MAP_FAILED,
            // but it is not defined in ctypes for an unknown reason
            return Ok(-1 as _);
        }
        Ok::<*mut c_void, LinuxError>(-1 as _)
    })
}

/// give advice about use of memory
/// if success return 0, if error return -1
///
/// TODO: implement this
pub fn sys_madvice(addr: *mut c_void, len: ctypes::size_t, advice: c_int) -> c_int {
    debug!(
        "sys_madvice <= addr: {:p}, len: {}, advice: {}",
        addr, len, advice
    );
    syscall_body!(sys_madvice, Ok(0))
}
