/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use core::ffi::{c_int, c_void};

use ruxos_posix_api::{sys_madvise, sys_mmap, sys_mprotect, sys_mremap, sys_msync, sys_munmap};

/// Map a file or device into virtual memory.
#[no_mangle]
pub unsafe extern "C" fn mmap(
    addr: *mut c_void,
    len: ctypes::size_t,
    prot: c_int,
    flags: c_int,
    fid: c_int,
    offset: ctypes::off_t,
) -> *mut c_void {
    sys_mmap(addr, len, prot, flags, fid, offset)
}

/// Unmap a range address of memory.
#[no_mangle]
pub unsafe extern "C" fn munmap(addr: *mut c_void, len: ctypes::size_t) -> c_int {
    sys_munmap(addr, len)
}

/// Sync pages mapped in memory to file.
#[no_mangle]
pub unsafe extern "C" fn msync(addr: *mut c_void, len: ctypes::size_t, flags: c_int) -> c_int {
    sys_msync(addr, len, flags)
}

/// Remap the address for already mapped memory.
#[no_mangle]
pub unsafe extern "C" fn mremap(
    old_addr: *mut c_void,
    old_size: ctypes::size_t,
    new_size: ctypes::size_t,
    flags: c_int,
    new_addr: *mut c_void,
) -> *mut c_void {
    sys_mremap(old_addr, old_size, new_size, flags, new_addr)
}

/// Change the accessiblity for already mapped memory.
#[no_mangle]
pub unsafe extern "C" fn mprotect(addr: *mut c_void, len: ctypes::size_t, flags: c_int) -> c_int {
    sys_mprotect(addr, len, flags)
}

/// Advise the operating system about the expected behavior of a specific region of memory.
///
/// Note: Unimplement yet.
#[no_mangle]
pub unsafe extern "C" fn madvise(addr: *mut c_void, len: ctypes::size_t, advice: c_int) -> c_int {
    sys_madvise(addr, len, advice)
}
