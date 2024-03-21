/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use crate::ctypes;

#[cfg(feature = "fs")]
use {
    crate::imp::fs::{sys_open, sys_pread64, sys_pwrite64},
    core::ffi::c_char,
};

use alloc::{collections::BTreeMap, vec::Vec};
use axerrno::LinuxError;
use core::{
    cmp::{max, min},
    ffi::{c_int, c_void},
    ops::Bound,
    ptr::write_bytes,
};
use memory_addr::PAGE_SIZE_4K;
use page_table::{MappingFlags, PagingError};
use ruxhal::{
    mem::VirtAddr,
    paging::{
        alloc_page_preload, do_pte_map, pte_map_check, pte_query, pte_swap_preload, pte_unmap_page,
        pte_update_page,
    },
};
use spin::mutex::SpinMutex;

// use `used_fs` instead of `#[cfg(feature = "fs")]{}` to cancel the scope of code.
#[cfg(feature = "fs")]
macro_rules! used_fs {
    ($($code:tt)*) => {$($code)*};
}

#[cfg(not(feature = "fs"))]
macro_rules! used_fs {
    ($($code:tt)*) => {};
}

const VMA_START: usize = 0xffff_8000_0000_0000_usize;
const VMA_END: usize = 0xffff_ff00_0000_0000_usize;
used_fs! {
    const SWAP_MAX: usize = 1024 * 1024 * 1024;
    const SWAP_PATH: &str = "swap.raw\0";
    static SWAPED_MAP: SpinMutex<BTreeMap<usize, Offset>> = SpinMutex::new(BTreeMap::new()); // Vaddr => (page_size, offset_at_swaped)
    lazy_static::lazy_static! {
        static ref SWAP_FID: i32 = sys_open(SWAP_PATH.as_ptr() as *const c_char, (ctypes::O_RDWR| ctypes::O_TRUNC| ctypes::O_SYNC) as i32, 0);
        static ref BITMAP_FREE: SpinMutex<Vec<usize>> = SpinMutex::new((0..SWAP_MAX).step_by(PAGE_SIZE_4K).collect());
    }
}

static VMA_MAP: SpinMutex<BTreeMap<usize, Vma>> = SpinMutex::new(BTreeMap::new()); // start_addr
static MEM_MAP: SpinMutex<BTreeMap<usize, PageInfo>> = SpinMutex::new(BTreeMap::new()); // Vaddr => (fid, offset, page_size)

type PageInfo = Option<(Fid, Offset, Len)>; // (fid, offset, page_size)
type Offset = usize;
type Fid = i32;
type Len = usize;

/// Data structure for mapping [start_addr, end_addr) with meta data.
#[derive(Debug)]
struct Vma {
    start_addr: usize,
    end_addr: usize,
    fid: i32,
    offset: usize,
    prot: u32,
    flags: u32,
}

/// Impl for Vma.
impl Vma {
    fn new(fid: i32, offset: usize, prot: u32, flags: u32) -> Self {
        Vma {
            start_addr: 0,
            end_addr: 0,
            fid,
            offset,
            flags,
            prot,
        }
    }

    fn clone_from(vma: &Vma, start_addr: usize, end_addr: usize) -> Self {
        Vma {
            start_addr,
            end_addr,
            fid: vma.fid,
            offset: vma.offset,
            prot: vma.prot,
            flags: vma.prot,
        }
    }
}

/// transform usize-like mmap flags to MappingFlags
fn get_mflags_from_usize(prot: u32) -> MappingFlags {
    let mut mmap_prot = MappingFlags::empty();

    if prot & ctypes::PROT_WRITE != 0 {
        mmap_prot |= MappingFlags::WRITE;
    }
    if prot & ctypes::PROT_EXEC != 0 {
        mmap_prot |= MappingFlags::EXECUTE;
    }

    // always readable at least
    mmap_prot | MappingFlags::READ
}

/// lock overlap region between two intervals [start1, end1) and [start2,end2)。
fn get_overlap(interval1: (usize, usize), interval2: (usize, usize)) -> Option<(usize, usize)> {
    let (start1, end1) = interval1;
    let (start2, end2) = interval2;

    let overlap_start = max(start1, start2);
    let overlap_end = min(end1, end2);

    if overlap_end > overlap_start {
        Some((overlap_start, overlap_end))
    } else {
        None
    }
}

/// search a free region in `VMA_LIST` meet the condition.
/// take care of AA-deadlock, this function should not be used after `MEM_MAP` is used.
fn find_free_region(
    vma_map: &BTreeMap<usize, Vma>,
    addr: Option<usize>,
    len: usize,
) -> Option<usize> {
    // Search free region in select region if start!=NULL, return error if `MAP_FIXED` flags exist.
    if let Some(start) = addr {
        let end_addr = if let Some(lower_vma) = vma_map.upper_bound(Bound::Included(&start)).value()
        {
            lower_vma.end_addr
        } else {
            VMA_START
        };
        let upper = vma_map
            .lower_bound(Bound::Included(&start))
            .key()
            .unwrap_or(&VMA_END);
        if upper - start >= len && end_addr <= start {
            return Some(start);
        }
    }

    // Search free region on the top of VMA_LISTS first.
    if let Some((_, last_vma)) = vma_map.last_key_value() {
        if VMA_END - last_vma.end_addr >= len {
            return Some(last_vma.end_addr);
        }
    } else if VMA_END >= VMA_START + len {
        return Some(VMA_START);
    }

    // Search free region among the VMA_LISTS.
    let mut left = VMA_START;
    for vma in vma_map.values() {
        let right = vma.start_addr;
        let free_size = right - left;
        if free_size >= len {
            return Some(left);
        }
        left = vma.end_addr;
    }

    None
}

/// release the range of [start, end) in mem_map
/// take care of AA-deadlock, this function should not be used after `MEM_MAP` is used.
fn release_pages_mapped(start: usize, end: usize) {
    let mut memory_map = MEM_MAP.lock();
    let mut removing_vaddr = Vec::new();
    for (&vaddr, &page_info) in memory_map.range(start..end) {
        #[cfg(feature = "fs")]
        if let Some((fid, offset, size)) = page_info {
            let src = vaddr as *mut c_void;
            let offset = offset as i64;
            sys_pwrite64(fid, src, size, offset);
        }
        if pte_unmap_page(VirtAddr::from(vaddr)).is_err() {
            panic!("Release page failed when munmapping!");
        }
        removing_vaddr.push(vaddr);
    }
    for vaddr in removing_vaddr {
        memory_map.remove(&vaddr);
    }
}

/// release the range of [start, end) in swaped-file, swaped-file should not contain file-mapping.
/// take care of AA-deadlock, this function should not be used after `SWAPED_MAP` and `BITMAP_FREE` is used.
#[cfg(feature = "fs")]
fn release_pages_swaped(start: usize, end: usize) {
    let mut swap_map = SWAPED_MAP.lock();
    let mut off_pool = BITMAP_FREE.lock();

    let mut removing_vaddr = Vec::new();
    for (&vaddr, &off) in swap_map.range(start..end) {
        removing_vaddr.push(vaddr);
        off_pool.push(off);
    }
    for vaddr in removing_vaddr {
        swap_map.remove(&vaddr);
    }
}

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl ruxhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_page_fault(vaddr: usize, cause: i64) -> i64 {
        let vma_map = VMA_MAP.lock();
        if pte_map_check(VirtAddr::from(vaddr)).is_ok() {
            return 0;
        }
        if let Some(vma) = vma_map.upper_bound(Bound::Included(&vaddr)).value() {
            if vma.end_addr < vaddr {
                error!(
                    "mapped={:x?},vaddr={:x?},cause={:x?}",
                    vma_map, vaddr, cause
                );
                return LinuxError::EFAULT as i64;
            }
            let vaddr = VirtAddr::from(vaddr).align_down_4k().as_usize();
            let size = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
            let map_flag = get_mflags_from_usize(vma.prot);

            trace!(
                "Page Fault Happening, vaddr:{:?}, casue:{}, map_flags:{:x?}",
                vaddr,
                cause,
                map_flag
            );
            // check if the access meet the prot
            if (cause == -1 && !map_flag.contains(MappingFlags::EXECUTE))
                || (cause == 0 && !map_flag.contains(MappingFlags::READ))
                || (cause == 1 && !map_flag.contains(MappingFlags::WRITE))
            {
                return LinuxError::EACCES as i64;
            }

            let mut memory_table = MEM_MAP.lock();
            used_fs! {
                let mut swaped_table = SWAPED_MAP.lock();
                let mut off_pool = BITMAP_FREE.lock();
            }
            // upload the page into PTE, and put file content inside.
            let fake_vaddr = match alloc_page_preload() {
                Ok(vaddr) => vaddr,
                Err(PagingError::NoMemory) if cfg!(feature = "fs") => {
                    used_fs! {
                        match memory_table.pop_first() {
                            Some((vaddr_swapped, Some((fid, offset, size)))) => {
                                let offset = offset.try_into().unwrap();
                                sys_pwrite64(fid, vaddr_swapped as *mut c_void, size, offset);
                                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                            }
                            Some((vaddr_swapped, None)) => {
                                let offset_get = off_pool.pop();
                                if SWAP_FID.is_negative() || offset_get.is_none() {
                                    panic!(
                                        "No free memory for mmap or swap fid, swap fid={}",
                                        *SWAP_FID
                                    );
                                }
                                let offset = offset_get.unwrap();
                                swaped_table.insert(vaddr_swapped, offset);

                                sys_pwrite64(
                                    *SWAP_FID,
                                    vaddr_swapped as *mut c_void,
                                    PAGE_SIZE_4K,
                                    offset as i64,
                                );

                                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                            }
                            _ => panic!("No memory for mmap, check if huge memory leaky exists"),
                        }
                    }
                    #[cfg(not(feature = "fs"))]
                    // this code will never be executed
                    panic!("No memory for mmap, check if huge memory leaky exists");
                }
                Err(ecode) => panic!(
                    "Unexpected error {:x?} happening when page fault occurs!",
                    ecode
                ),
            };

            if vma.fid > 0 && !map_flag.is_empty() {
                used_fs! {
                    let dst = fake_vaddr.as_mut_ptr() as *mut c_void;
                    let (write_fid, off) = if let Some(off) = swaped_table.remove(&vaddr) {
                        off_pool.push(off);
                        (*SWAP_FID, off as i64)
                    } else {
                        (vma.fid, (vma.offset + (vaddr - vma.start_addr)) as i64)
                    };
                    sys_pread64(write_fid, dst, size, off);
                }
            } else {
                let dst = fake_vaddr.as_mut_ptr() as *mut c_void;
                unsafe {
                    write_bytes(dst, 0, size);
                }
            }

            if (vma.prot & ctypes::PROT_WRITE != 0)
                && (vma.flags & ctypes::MAP_PRIVATE == 0)
                && (vma.fid > 0)
            {
                let map_length = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
                let offset = vma.offset + (vaddr - vma.start_addr);
                memory_table.insert(vaddr, Some((vma.fid, offset, map_length)));
            } else {
                memory_table.insert(vaddr, None);
            }

            match do_pte_map(VirtAddr::from(vaddr), fake_vaddr, map_flag) {
                Ok(()) => 0,
                Err(_) => LinuxError::EFAULT as i64,
            }
        } else {
            for mapped in vma_map.iter() {
                warn!("{:x?}", mapped);
            }
            warn!("vaddr={:x?},cause={:x?}", vaddr, cause);
            LinuxError::EFAULT as i64
        }
    }
}

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
        "sys_mmap <= start: {:p}, len: {:x}, prot:{:x?}, flags:{:x?}, fd: {}",
        start, len, prot, flags, fd
    );
    syscall_body!(sys_mmap, {
        // transform C-type into rust-type
        let start = start as usize;
        let len = VirtAddr::from(len).align_up_4k().as_usize();
        if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
            return Err(LinuxError::EINVAL);
        }
        let prot = prot as u32;
        let flags = flags as u32;
        let fid = fd;
        let offset = off as usize;

        // check if `MAP_SHARED` or `MAP_PRIVATE` within flags.
        if (flags & ctypes::MAP_PRIVATE == 0) && (flags & ctypes::MAP_SHARED == 0) {
            return Err(LinuxError::EINVAL);
        }

        // check if `MAP_ANOYMOUS` within flags.
        let fid = if flags & ctypes::MAP_ANONYMOUS != 0 {
            -1
        } else if fid < 0 {
            error!("fd in mmap without `MAP_ANONYMOUS` must larger than 0");
            return Err(LinuxError::EINVAL);
        } else {
            fid
        };

        let mut new = Vma::new(fid, offset, prot, flags);
        let mut vma_map = VMA_MAP.lock();
        let addr_condition = if start == 0 { None } else { Some(start) };
        let try_addr = find_free_region(&vma_map, addr_condition, len);
        match try_addr {
            Some(vaddr) if vaddr == start || flags & ctypes::MAP_FIXED == 0 => {
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
    debug!("sys_munmap <= start: {:p}, len: {:x}", start, len);
    syscall_body!(sys_munmap, {
        // transform C-type into rust-type
        let start = start as usize;
        let end = VirtAddr::from(start + len).align_up_4k().as_usize();

        if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K)
            || !VirtAddr::from(len).is_aligned(PAGE_SIZE_4K)
            || len == 0
        {
            error!(
                "sys_munmap start_address={:x}, len {:x?} not aligned",
                start, len
            );
            return Err(LinuxError::EINVAL);
        }

        let mut vma_map = VMA_MAP.lock();
        let mut post_append: Vec<(usize, Vma)> = Vec::new(); // vma should be insert.
        let mut post_remove: Vec<usize> = Vec::new(); // vma should be removed.
        let mut node = vma_map.upper_bound_mut(Bound::Included(&start));
        let mut counter = 0; // counter to check if all address in [start, start+len) is mapped.
        while let Some(vma) = node.value_mut() {
            if vma.start_addr > end {
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
                    vma.end_addr = overlapped_start;
                } else {
                    post_remove.push(vma.start_addr);
                }
            }
            node.move_next();
        }

        // check if any address in [start, end) not mayed.
        if counter != end - start {
            error!(
                "munmap {:x?} but only {:x?} Byte inside",
                end - start,
                counter
            );
            return Err(LinuxError::EINVAL);
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
/// containing any part of the address range in the interval [addr, addr+len-1].  
/// addr must be aligned to a page boundary.
pub fn sys_mprotect(start: *mut c_void, len: ctypes::size_t, prot: c_int) -> c_int {
    debug!(
        "sys_mprotect <= addr: {:p}, len: {:x}, prot: {}",
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

        let mut post_append: Vec<(usize, Vma)> = Vec::new();

        let mut vma_map = VMA_MAP.lock();
        let mut node = vma_map.upper_bound_mut(Bound::Included(&start));
        while let Some(vma) = node.value_mut() {
            if vma.start_addr > end {
                break;
            }
            if let Some((overlapped_start, overlapped_end)) =
                get_overlap(syncing_interval, (vma.start_addr, vma.end_addr))
            {
                for (&vaddr, _) in MEM_MAP.lock().range(overlapped_start..overlapped_end) {
                    if pte_update_page(
                        VirtAddr::from(vaddr),
                        None,
                        Some(get_mflags_from_usize(prot as u32)),
                    )
                    .is_err()
                    {
                        error!(
                                "Update page prot failed when mprotecting the page: vaddr={:x?}, prot={:?}",
                                vaddr, prot
                            );
                    }
                }
                // add node for overlapped vma_ptr
                if vma.end_addr > overlapped_end {
                    let right_vma = Vma::clone_from(vma, overlapped_end, vma.end_addr);
                    post_append.push((overlapped_end, right_vma));
                }
                if overlapped_start > vma.start_addr {
                    vma.end_addr = overlapped_start;
                    let mut overlapped_vma = Vma::clone_from(vma, overlapped_start, overlapped_end);
                    overlapped_vma.prot = prot as u32;
                    post_append.push((overlapped_start, overlapped_vma))
                } else {
                    vma.end_addr = overlapped_end;
                    vma.prot = prot as u32;
                }
            }
            node.move_next();
        }

        for (key, value) in post_append {
            vma_map.insert(key, value);
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
        used_fs! {
            let start = start as usize;
            let end = VirtAddr::from(start + len).align_up_4k().as_usize();
            if !VirtAddr::from(start).is_aligned(PAGE_SIZE_4K) || len == 0 {
                return Err(LinuxError::EINVAL);
            }
            for (&vaddr, &page_info) in MEM_MAP.lock().range(start..end) {
                if let Some((fid, offset, size)) = page_info {
                    let src = vaddr as *mut c_void;
                    let ret_size = sys_pwrite64(fid, src, size, offset as i64) as usize;
                    if size != ret_size {
                        error!(
                            "sys_msync: try to pwrite(fid={:x?}, size={:x?}, offset={:x?}) but get ret = {:x?}",
                            fid, size, offset, ret_size
                        );
                        return Err(LinuxError::EFAULT);
                    }
                }
            }
        }
        Ok(0)
    })
}

/// shift mapped the page in both MEM_MAP and SWAPED_MAP.
/// No page fault here should be guaranteed
fn shift_mapped_page(start: usize, end: usize, vma_offset: usize, copy: bool) {
    let mut memory_table = MEM_MAP.lock();
    used_fs! {
        let mut swaped_table = SWAPED_MAP.lock();
        let mut off_pool = BITMAP_FREE.lock();
    }

    let mut opt_buffer = Vec::new();
    for (&start, &page_info) in memory_table.range(start..end) {
        opt_buffer.push((start, page_info));
    }
    for (start, page_info) in opt_buffer {
        // opt for the PTE.
        let (fake_vaddr, flags) = if !copy {
            memory_table.remove(&start);
            // only shift virtual address and keep the physic address to free from data-copy.
            let (_, flags, _) = pte_query(VirtAddr::from(start)).unwrap();
            let fake_vaddr = pte_swap_preload(VirtAddr::from(start)).unwrap();
            (fake_vaddr, flags)
        } else {
            let (_, flags, _) = pte_query(VirtAddr::from(start)).unwrap();
            let fake_vaddr = match alloc_page_preload() {
                Ok(vaddr) => vaddr,
                Err(PagingError::NoMemory) if cfg!(feature = "fs") => {
                    used_fs! {
                        match memory_table.pop_first() {
                            Some((vaddr_swapped, Some((fid, offset, size)))) => {
                                let offset = offset.try_into().unwrap();
                                sys_pwrite64(fid, vaddr_swapped as *mut c_void, size, offset);
                                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                            }
                            Some((vaddr_swapped, None)) => {
                                let offset_get = off_pool.pop();
                                if SWAP_FID.is_negative() || offset_get.is_none() {
                                    panic!(
                                        "No free memory for mmap or swap fid, swap fid={}",
                                        *SWAP_FID
                                    );
                                }
                                let offset = offset_get.unwrap();
                                swaped_table.insert(vaddr_swapped, offset);

                                sys_pwrite64(
                                    *SWAP_FID,
                                    vaddr_swapped as *mut c_void,
                                    PAGE_SIZE_4K,
                                    offset as i64,
                                );

                                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                            }
                            _ => panic!("No memory for mmap, check if huge memory leaky exists"),
                        }
                    }
                    #[cfg(not(feature = "fs"))]
                    panic!("No memory for mmap, check if huge memory leaky exists");
                    // this code will never be executed
                }
                Err(ecode) => panic!(
                    "Unexpected error {:x?} happening when page fault occurs!",
                    ecode
                ),
            };
            let dst = unsafe {
                core::slice::from_raw_parts_mut(fake_vaddr.as_usize() as *mut u8, PAGE_SIZE_4K)
            };
            if memory_table.contains_key(&start) {
                // copy the memory directly
                let src =
                    unsafe { core::slice::from_raw_parts_mut(start as *mut u8, PAGE_SIZE_4K) };
                dst.clone_from_slice(src);
            } else if page_info.is_none()
            /* has been swapped from memory */
            {
                used_fs! {
                    let offset = swaped_table.get(&start).unwrap();
                    sys_pread64(
                        *SWAP_FID,
                        dst.as_mut_ptr() as *mut c_void,
                        PAGE_SIZE_4K,
                        *offset as i64,
                    );
                }
            }
            (fake_vaddr, flags)
        };
        do_pte_map(VirtAddr::from(start + vma_offset), fake_vaddr, flags).unwrap();
        memory_table.insert(start + vma_offset, page_info);
    }

    used_fs! {
        let mut opt_buffer = Vec::new();
        for (&start, &off_in_swap) in swaped_table.range(start..end) {
            opt_buffer.push((start, off_in_swap));
        }
        for (start, off_in_swap) in opt_buffer {
            // opt for the swapped file, should copy swaped page for the new page.
            if !copy {
                swaped_table.remove(&start);
            } else {
                let off_ptr = off_pool
                    .pop()
                    .expect("There are no free space in swap-file!");
                let mut rw_buffer: [u8; PAGE_SIZE_4K] = [0_u8; PAGE_SIZE_4K];
                sys_pread64(
                    *SWAP_FID,
                    rw_buffer.as_mut_ptr() as *mut c_void,
                    PAGE_SIZE_4K,
                    off_in_swap as i64,
                );
                sys_pwrite64(
                    *SWAP_FID,
                    rw_buffer.as_mut_ptr() as *mut c_void,
                    PAGE_SIZE_4K,
                    off_ptr as i64,
                );
            }
            swaped_table.insert(start + vma_offset, off_in_swap);
        }
    }
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
        let mut consistent_vma: Option<Vma> = None;

        let mut post_remove: Vec<usize> = Vec::new(); // vma should be removed.

        let mut vma_map = VMA_MAP.lock();
        // collect and check vma alongside the range of [old_start, old_end).
        let mut node = vma_map.upper_bound_mut(Bound::Included(&old_start));
        while let Some(vma) = node.value_mut() {
            if vma.start_addr > old_end {
                break;
            }
            // make sure of consistent_vma is continuous and contistence in both flags and prot.
            if let Some(ref mut inner_vma) = consistent_vma {
                if inner_vma.end_addr == vma.start_addr
                    && inner_vma.flags == vma.flags
                    && inner_vma.prot == vma.prot
                    && inner_vma.fid == vma.fid
                    && inner_vma.offset + (inner_vma.end_addr - inner_vma.start_addr) == vma.offset
                {
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
