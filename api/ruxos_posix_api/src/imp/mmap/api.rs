/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use alloc::vec::Vec;
use axerrno::LinuxError;
use core::{
    ffi::{c_int, c_void},
    ops::Bound,
};
use memory_addr::PAGE_SIZE_4K;
use ruxhal::mem::VirtAddr;
use ruxmm::paging::pte_update_page;

use super::utils::{
    find_free_region, get_mflags_from_usize, get_overlap, release_pages_mapped, shift_mapped_page,
    snatch_fixed_region, VMA_END,
};
use ruxtask::vma::Vma;
use ruxtask::{current, vma::FileInfo};

#[cfg(feature = "fs")]
use {
    super::utils::{release_pages_swaped, write_into},
    alloc::sync::Arc,
};

/// Creates a new mapping in the virtual address space of the calling process.
///
/// Note: support flags `MAP_PRIVATE`, `MAP_SHARED`, `MAP_ANONYMOUS`, `MAP_FILE`, `MAP_FIXED`.
pub fn sys_mmap(
    start: *mut c_void,
    len: ctypes::size_t,
    prot: c_int,
    flags: c_int,
    fd: c_int,
    off: ctypes::off_t,
) -> *mut c_void {
    debug!(
        "sys_mmap <= start: {:p}, len: 0x{:x}, prot:0x{:x?}, flags:0x{:x?}, fd: {}",
        start, len, prot, flags, fd
    );
    syscall_body!(sys_mmap, {
        // transform C-type into rust-type
        let start = start as usize;
        let len = VirtAddr::from(len).align_up_4k().as_usize();
        if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
            error!(
                "mmap failed because start:0x{:x} is not aligned or len:0x{:x} == 0",
                start, len
            );
            return Err(LinuxError::EINVAL);
        }
        let prot = prot as u32;
        let flags = flags as u32;
        let fid = fd;
        let offset = off as usize;

        // check if `MAP_SHARED` or `MAP_PRIVATE` within flags.
        if (flags & ctypes::MAP_PRIVATE == 0) && (flags & ctypes::MAP_SHARED == 0) {
            error!("mmap failed because none of `MAP_PRIVATE` and `MAP_SHARED` exist");
            return Err(LinuxError::EINVAL);
        }

        // check if `MAP_ANOYMOUS` within flags.
        let fid = if flags & ctypes::MAP_ANONYMOUS != 0 {
            -1
        } else if fid < 0 {
            error!("fd in mmap without `MAP_ANONYMOUS` must larger than 0");
            return Err(LinuxError::EBADF);
        } else {
            fid
        };

        let mut new = Vma::new(fid, offset, prot, flags);
        let binding_task = current();
        let mut vma_map = binding_task.mm.vma_map.lock();
        let addr_condition = if start == 0 { None } else { Some(start) };

        let try_addr = if flags & ctypes::MAP_FIXED != 0 {
            snatch_fixed_region(&mut vma_map, start, len)
        } else {
            find_free_region(&vma_map, addr_condition, len)
        };

        match try_addr {
            Some(vaddr) => {
                new.start_addr = vaddr;
                new.end_addr = vaddr + len;
                vma_map.insert(vaddr, new);
                Ok(vaddr as *mut c_void)
            }
            _ => Err(LinuxError::ENOMEM),
        }
    })
}

/// Deletes the mappings for the specified address range
pub fn sys_munmap(start: *mut c_void, len: ctypes::size_t) -> c_int {
    debug!("sys_munmap <= start: {:p}, len: 0x{:x}", start, len);
    syscall_body!(sys_munmap, {
        // transform C-type into rust-type
        let start = start as usize;
        let end = VirtAddr::from(start + len).align_up_4k().as_usize();

        if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
            error!(
                "sys_munmap start_address=0x{:x}, len 0x{:x?} not aligned",
                start, len
            );
            return Err(LinuxError::EINVAL);
        }

        let binding = current();
        let mut vma_map = binding.mm.vma_map.lock();

        // In order to ensure that munmap can exit directly if it fails, it must
        // ensure that munmap semantics are correct before taking action.
        let mut post_shrink: Vec<(usize, usize)> = Vec::new(); // vma should be insert.
        let mut post_append: Vec<(usize, Vma)> = Vec::new(); // vma should be insert.
        let mut post_remove: Vec<usize> = Vec::new(); // vma should be removed.

        let mut node = vma_map.upper_bound_mut(Bound::Included(&start));
        let mut counter = 0; // counter to check if all address in [start, start+len) is mapped.
        while let Some(vma) = node.value_mut() {
            if vma.start_addr >= end {
                break;
            }
            if let Some((overlapped_start, overlapped_end)) =
                get_overlap((start, end), (vma.start_addr, vma.end_addr))
            {
                // Accumulate the size of the mapping area to be released
                counter += overlapped_end - overlapped_start;

                // add node for overlapped vma_ptr
                if vma.end_addr > overlapped_end {
                    let right_vma = Vma::clone_from(vma, overlapped_end, vma.end_addr);
                    post_append.push((overlapped_end, right_vma));
                }
                if overlapped_start > vma.start_addr {
                    // do vma.end_addr = overlapped_start if success
                    post_shrink.push((vma.start_addr, overlapped_start));
                } else {
                    post_remove.push(vma.start_addr);
                }
            }
            node.move_next();
        }

        // check if any address in [start, end) not mayed.
        if counter != end - start {
            error!(
                "munmap 0x{:x?} but only 0x{:x?} byte inside",
                end - start,
                counter
            );
            return Err(LinuxError::EINVAL);
        }

        // do action after success.
        for (start, addr) in post_shrink {
            let vma_shrinking = vma_map.get_mut(&start).unwrap();
            vma_shrinking.end_addr = addr;
        }
        for key in post_remove {
            vma_map.remove(&key).expect("there should be no empty");
        }
        for (key, value) in post_append {
            vma_map.insert(key, value);
        }

        // delete the mapped and swapped page.
        release_pages_mapped(start, end);
        #[cfg(feature = "fs")]
        release_pages_swaped(start, end);

        Ok(0)
    })
}

/// Changes the access protections for the calling process's memory pages
/// containing any part of the address range in the interval [addr, addr+len).  
/// addr must be aligned to a page boundary.
pub fn sys_mprotect(start: *mut c_void, len: ctypes::size_t, prot: c_int) -> c_int {
    debug!(
        "sys_mprotect <= addr: {:p}, len: 0x{:x}, prot: {}",
        start, len, prot
    );

    syscall_body!(sys_mprotect, {
        // transform C-type into rust-type
        let start = start as usize;
        let end = VirtAddr::from(start + len).align_up_4k().as_usize();
        if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
            return Err(LinuxError::EINVAL);
        }
        let syncing_interval = (start, end);

        // In order to ensure that munmap can exit directly if it fails, it must
        // ensure that munmap semantics are correct before taking action.
        let mut post_append: Vec<(usize, Vma)> = Vec::new();
        let mut post_shrink: Vec<(usize, usize)> = Vec::new();
        let mut post_align_changed: Vec<(usize, usize)> = Vec::new();

        let binding_task = current();
        let mut vma_map = binding_task.mm.vma_map.lock();
        let mut node = vma_map.upper_bound_mut(Bound::Included(&start));
        let mut counter = 0; // counter to check if all address in [start, start+len) is mapped.
        while let Some(vma) = node.value_mut() {
            if vma.start_addr >= end {
                break;
            }
            if let Some((overlapped_start, overlapped_end)) =
                get_overlap(syncing_interval, (vma.start_addr, vma.end_addr))
            {
                // Accumulate the size of the mapping area to be released
                counter += overlapped_end - overlapped_start;

                // add node for overlapped vma_ptr
                if vma.end_addr > overlapped_end {
                    let right_vma = Vma::clone_from(vma, overlapped_end, vma.end_addr);
                    post_append.push((overlapped_end, right_vma));
                }
                if overlapped_start > vma.start_addr {
                    // do vma.end_addr = overlapped_start if success
                    post_shrink.push((vma.start_addr, overlapped_start));

                    // The left side of the vma needs to be kept as is, while `prot`
                    // on the right side need to be modified.
                    let mut overlapped_vma = Vma::clone_from(vma, overlapped_start, overlapped_end);
                    overlapped_vma.prot = prot as u32;
                    post_append.push((overlapped_start, overlapped_vma))
                } else {
                    // do vma.end_addr = overlapped_end and vma.prot = prot as u32 if success
                    post_align_changed.push((vma.start_addr, overlapped_end));
                }
            }
            node.move_next();
        }
        // check if any address in [start, end) not mayed.
        if counter != end - start {
            error!(
                "mprotect 0x{:x?} but only 0x{:x?} byte inside",
                end - start,
                counter
            );
            return Err(LinuxError::EFAULT);
        }

        // upate PTEs if mprotect is successful.
        for (&vaddr, _) in current().mm.mem_map.lock().range(start..end) {
            if pte_update_page(
                VirtAddr::from(vaddr),
                None,
                Some(get_mflags_from_usize(prot as u32)),
            )
            .is_err()
            {
                error!(
                    "Updating page prot failed when mprotecting the page: vaddr=0x{:x?}, prot={:?}",
                    vaddr, prot
                );
            }
        }
        // do action after success.
        for (key, value) in post_append {
            vma_map.insert(key, value);
        }
        for (start, addr) in post_shrink {
            let vma_shrinking = vma_map.get_mut(&start).unwrap();
            vma_shrinking.end_addr = addr;
        }
        for (start, addr) in post_align_changed {
            let vma_align_changing = vma_map.get_mut(&start).unwrap();
            vma_align_changing.end_addr = addr;
            vma_align_changing.prot = prot as u32;
        }
        Ok(0)
    })
}

/// Synchronizes the calling process's memory pages in the interval [addr, addr+len-1]
/// with the corresponding physical storage device, ensuring that any modifications
/// are flushed to the storage.
///
/// Note: support flags `MS_SYNC` only.
pub fn sys_msync(start: *mut c_void, len: ctypes::size_t, flags: c_int) -> c_int {
    debug!(
        "sys_msync <= addr: {:p}, len: {}, flags: {}",
        start, len, flags
    );
    syscall_body!(sys_msync, {
        #[cfg(feature = "fs")]
        {
            let start = start as usize;
            let end = VirtAddr::from(start + len).align_up_4k().as_usize();
            if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
                return Err(LinuxError::EINVAL);
            }
            for (&vaddr, page_info) in current().mm.mem_map.lock().range(start..end) {
                if let Some(FileInfo { file, offset, size }) = &page_info.mapping_file {
                    let src = vaddr as *mut u8;
                    write_into(&file, src, *offset as u64, *size);
                }
            }
        }
        Ok(0)
    })
}

/// Remap a virtual memory address
pub fn sys_mremap(
    old_addr: *mut c_void,
    old_size: ctypes::size_t,
    new_size: ctypes::size_t,
    flags: c_int,
    new_addr: *mut c_void,
) -> *mut c_void {
    debug!(
        "sys_mremap <= old_addr: {:p}, old_size: {}, new_size: {}, flags: {}, new_addr: {:p}",
        old_addr, old_size, new_size, flags, new_addr
    );
    syscall_body!(sys_mremap, {
        let old_vaddr = VirtAddr::from(old_addr as usize);
        let flags = flags as u32;

        // check if the parameters is legal.
        if (!old_vaddr.is_aligned(PAGE_SIZE_4K))
            || new_size == 0
            || (old_size == 0 && flags & ctypes::MREMAP_MAYMOVE == 0)
            || (old_size != new_size && flags & ctypes::MREMAP_DONTUNMAP != 0)
            || flags & !(ctypes::MREMAP_MAYMOVE | ctypes::MREMAP_FIXED | ctypes::MREMAP_DONTUNMAP)
                != 0
            || ((flags & ctypes::MREMAP_FIXED != 0 || flags & ctypes::MREMAP_DONTUNMAP != 0)
                && (flags & ctypes::MREMAP_MAYMOVE == 0))
        {
            return Err(LinuxError::EINVAL);
        }

        let old_start = old_vaddr.as_usize();
        let old_end = old_start + old_size;

        let mut consistent_vma: Option<Vma> = None; // structure to verify the consistent in the range of [old_start, old_end)
        let mut post_remove: Vec<usize> = Vec::new(); // vma should be removed if success.

        let binding_task = current();
        let mut vma_map = binding_task.mm.vma_map.lock();
        // collect and check vma alongside the range of [old_start, old_end).
        let mut node = vma_map.upper_bound_mut(Bound::Included(&old_start));
        while let Some(vma) = node.value_mut() {
            if vma.start_addr > old_end {
                break;
            }
            // make sure of consistent_vma is continuous and consistent in both flags and prots.
            if let Some(ref mut inner_vma) = consistent_vma {
                if inner_vma.end_addr == vma.start_addr
                    && inner_vma.flags == vma.flags
                    && inner_vma.prot == vma.prot
                {
                    #[cfg(feature = "fs")]
                    if inner_vma.file.is_some() {
                        if vma.file.is_none() {
                            return Err(LinuxError::EFAULT);
                        }
                        let end_offset =
                            inner_vma.offset + (inner_vma.end_addr - inner_vma.start_addr);
                        let vma_file = vma.file.as_ref().unwrap();
                        let inner_file = inner_vma.file.as_ref().unwrap();
                        if !Arc::ptr_eq(vma_file, inner_file) || end_offset != vma.offset {
                            return Err(LinuxError::EFAULT);
                        }
                    } else if vma.file.is_some() {
                        return Err(LinuxError::EFAULT);
                    }
                    inner_vma.end_addr = vma.end_addr;
                } else {
                    return Err(LinuxError::EFAULT);
                }
            } else {
                consistent_vma.replace(Vma::clone_from(vma, vma.start_addr, vma.end_addr));
            }

            post_remove.push(vma.start_addr);
            node.move_next();
        }

        // check if consistent_vma full match the remapping memory.
        if consistent_vma.is_none() {
            return Err(LinuxError::EFAULT);
        }
        let mut old_vma = consistent_vma.unwrap();
        if old_vma.end_addr < old_end {
            return Err(LinuxError::EFAULT);
        }

        let opt_address = if flags & ctypes::MREMAP_FIXED != 0 || !new_addr.is_null() {
            Some(new_addr as usize)
        } else {
            None
        };

        if flags & ctypes::MREMAP_DONTUNMAP != 0 {
            // find a new region for new_start
            if let Some(new_start) = find_free_region(&vma_map, opt_address, new_size) {
                if flags & ctypes::MREMAP_FIXED != 0 && new_addr as usize != new_start {
                    return Err(LinuxError::ENOMEM);
                }

                // copy the dirty page.
                shift_mapped_page(
                    old_vma.start_addr,
                    old_vma.end_addr,
                    new_start - old_vma.start_addr,
                    true,
                );

                // copy the old to the new.
                vma_map.insert(
                    new_start,
                    Vma::clone_from(&old_vma, new_start, new_start + new_size),
                );

                // Remove the mapping debris and combine them into a large one.(for performance)
                for key in post_remove {
                    vma_map.remove(&key);
                }
                vma_map.insert(old_vma.start_addr, old_vma);

                return Ok(new_start as *mut c_void);
            } else {
                return Err(LinuxError::ENOMEM);
            }
        }

        // shrinking the original address does not require changing the mapped page.
        if old_size > new_size && (flags & ctypes::MREMAP_FIXED == 0 || new_addr == old_addr) {
            let ret = old_vma.start_addr;
            let new_end = old_start + new_size;
            if old_vma.end_addr > old_end {
                vma_map.insert(
                    old_end,
                    Vma::clone_from(&old_vma, old_end, old_vma.end_addr),
                );
            }
            old_vma.end_addr = new_end;

            // delete the mapped and swapped page outside of new vma.
            release_pages_mapped(new_end, old_end);
            #[cfg(feature = "fs")]
            release_pages_swaped(new_end, old_end);

            // vma_map.insert(old_vma.start_addr, old_vma);
            return Ok(ret as *mut c_void);
        }
        // expanding the original address does not require changing the mapped page.
        else if old_size < new_size && (flags & ctypes::MREMAP_FIXED == 0 || new_addr == old_addr)
        {
            if old_vma.end_addr != old_end && flags & ctypes::MREMAP_MAYMOVE == 0 {
                return Err(LinuxError::ENOMEM);
            }
            // find the right region to expand them in orignal addr.
            let upper = vma_map
                .lower_bound(Bound::Included(&old_end))
                .key()
                .unwrap_or(&VMA_END);
            if upper - old_end >= new_size - old_size {
                let ret = old_vma.start_addr;
                let new_end = old_start + new_size;
                old_vma.end_addr = new_end;
                vma_map.insert(old_vma.start_addr, old_vma);
                return Ok(ret as *mut c_void);
            }
        }

        // try to move pages according to `new_addr`.
        if flags & ctypes::MREMAP_MAYMOVE != 0 {
            match find_free_region(&vma_map, opt_address, new_size) {
                Some(vaddr) if vaddr == new_addr as usize || flags & ctypes::MREMAP_FIXED == 0 => {
                    // Reserve excess memory before and after
                    if old_vma.start_addr < old_start {
                        vma_map.insert(
                            old_vma.start_addr,
                            Vma::clone_from(&old_vma, old_vma.start_addr, old_start),
                        );
                    }
                    if old_vma.end_addr > old_end {
                        vma_map.insert(
                            old_end,
                            Vma::clone_from(&old_vma, old_end, old_vma.end_addr),
                        );
                    }

                    // Shift mapped memory in both `MEM_MAP` and `SWAP_MAP`.
                    shift_mapped_page(
                        old_vma.start_addr,
                        old_vma.end_addr,
                        vaddr - old_vma.start_addr,
                        false,
                    );

                    // Insert the new vma.
                    vma_map.insert(vaddr, Vma::clone_from(&old_vma, vaddr, vaddr + new_size));

                    // remove the old vma deris.
                    for key in post_remove {
                        vma_map.remove(&key);
                    }
                    if old_vma.start_addr < old_start {
                        vma_map.insert(
                            old_end,
                            Vma::clone_from(&old_vma, old_vma.start_addr, old_start),
                        );
                    }
                    if old_vma.end_addr > old_end {
                        vma_map.insert(
                            old_end,
                            Vma::clone_from(&old_vma, old_end, old_vma.end_addr),
                        );
                    }

                    return Ok(vaddr as *mut c_void);
                }
                _ => {
                    return Err(LinuxError::ENOMEM);
                }
            };
        }

        Err(LinuxError::ENOMEM)
    })
}

/// give advice about use of memory
/// if success return 0, if error return -1
///
/// TODO: implement this to improve performance.
pub fn sys_madvise(addr: *mut c_void, len: ctypes::size_t, advice: c_int) -> c_int {
    debug!(
        "sys_madvise <= addr: {:p}, len: {}, advice: {}",
        addr, len, advice
    );
    syscall_body!(sys_madvise, Ok(0))
}
