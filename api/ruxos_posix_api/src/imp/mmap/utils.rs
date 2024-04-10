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
    core::ffi::{c_char, c_void},
    page_table::PagingError,
};

use alloc::{collections::BTreeMap, vec::Vec};
use axsync::Mutex;
use core::{
    cmp::{max, min},
    ops::Bound,
};
use memory_addr::PAGE_SIZE_4K;
use page_table::MappingFlags;
use ruxhal::{
    mem::VirtAddr,
    paging::{alloc_page_preload, do_pte_map, pte_query, pte_swap_preload, pte_unmap_page},
};

// use `used_fs` instead of `#[cfg(feature = "fs")]{}` to cancel the scope of code.
#[cfg(feature = "fs")]
macro_rules! used_fs {
    ($($code:tt)*) => {$($code)*};
}

#[cfg(not(feature = "fs"))]
macro_rules! used_fs {
    ($($code:tt)*) => {};
}

pub(crate) const VMA_START: usize = ruxconfig::MMAP_START_VADDR;
pub(crate) const VMA_END: usize = ruxconfig::MMAP_END_VADDR;

// TODO: move defination of `SWAP_MAX` and `SWAP_PATH` from const numbers to `ruxconfig`.
used_fs! {
    pub(crate) const SWAP_MAX: usize = 1024 * 1024 * 1024;
    pub(crate) const SWAP_PATH: &str = "swap.raw\0";
    pub(crate) static SWAPED_MAP: Mutex<BTreeMap<usize, Offset>> = Mutex::new(BTreeMap::new()); // Vaddr => (page_size, offset_at_swaped)
    lazy_static::lazy_static! {
        pub(crate) static ref SWAP_FID: i32 = sys_open(SWAP_PATH.as_ptr() as *const c_char, (ctypes::O_RDWR| ctypes::O_TRUNC| ctypes::O_SYNC) as i32, 0);
        pub(crate) static ref BITMAP_FREE: Mutex<Vec<usize>> = Mutex::new((0..SWAP_MAX).step_by(PAGE_SIZE_4K).collect());
    }
}

pub(crate) static VMA_MAP: Mutex<BTreeMap<usize, Vma>> = Mutex::new(BTreeMap::new()); // start_addr
pub(crate) static MEM_MAP: Mutex<BTreeMap<usize, PageInfo>> = Mutex::new(BTreeMap::new()); // Vaddr => (fid, offset, page_size)

type PageInfo = Option<(Fid, Offset, Len)>; // (fid, offset, page_size)
type Offset = usize;
type Fid = i32;
type Len = usize;

/// Data structure for mapping [start_addr, end_addr) with meta data.
#[derive(Debug)]
pub(crate) struct Vma {
    pub start_addr: usize,
    pub end_addr: usize,
    pub fid: i32,
    pub offset: usize,
    pub prot: u32,
    pub flags: u32,
}

/// Impl for Vma.
impl Vma {
    pub fn new(fid: i32, offset: usize, prot: u32, flags: u32) -> Self {
        Vma {
            start_addr: 0,
            end_addr: 0,
            fid,
            offset,
            flags,
            prot,
        }
    }

    pub fn clone_from(vma: &Vma, start_addr: usize, end_addr: usize) -> Self {
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
pub(crate) fn get_mflags_from_usize(prot: u32) -> MappingFlags {
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
pub(crate) fn get_overlap(
    interval1: (usize, usize),
    interval2: (usize, usize),
) -> Option<(usize, usize)> {
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
pub(crate) fn find_free_region(
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
pub(crate) fn release_pages_mapped(start: usize, end: usize) {
    let mut memory_map = MEM_MAP.lock();
    let mut removing_vaddr = Vec::new();
    for (&vaddr, &_page_info) in memory_map.range(start..end) {
        #[cfg(feature = "fs")]
        if let Some((fid, offset, size)) = _page_info {
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
pub(crate) fn release_pages_swaped(start: usize, end: usize) {
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

/// shift mapped the page in both MEM_MAP and SWAPED_MAP.
/// No page fault here should be guaranteed
pub(crate) fn shift_mapped_page(start: usize, end: usize, vma_offset: usize, copy: bool) {
    let mut memory_map = MEM_MAP.lock();
    used_fs! {
        let mut swaped_map = SWAPED_MAP.lock();
        let mut off_pool = BITMAP_FREE.lock();
    }

    let mut opt_buffer = Vec::new();
    for (&start, &page_info) in memory_map.range(start..end) {
        opt_buffer.push((start, page_info));
    }
    for (start, page_info) in opt_buffer {
        // opt for the PTE.
        let (fake_vaddr, flags) = if !copy {
            memory_map.remove(&start);
            // only shift virtual address and keep the physic address to free from data-copy.
            let (_, flags, _) = pte_query(VirtAddr::from(start)).unwrap();
            let fake_vaddr = pte_swap_preload(VirtAddr::from(start)).unwrap();
            (fake_vaddr, flags)
        } else {
            let (_, flags, _) = pte_query(VirtAddr::from(start)).unwrap();

            #[cfg(not(feature = "fs"))]
            let fake_vaddr = alloc_page_preload().expect("alloc memory for new page failed");
            #[cfg(feature = "fs")]
            let fake_vaddr =
                preload_page_with_swap(&mut memory_map, &mut swaped_map, &mut off_pool);

            let dst = unsafe {
                core::slice::from_raw_parts_mut(fake_vaddr.as_usize() as *mut u8, PAGE_SIZE_4K)
            };
            if memory_map.contains_key(&start) {
                // copy the memory directly
                let src =
                    unsafe { core::slice::from_raw_parts_mut(start as *mut u8, PAGE_SIZE_4K) };
                dst.clone_from_slice(src);
            } else if page_info.is_none()
            /* has been swapped from memory */
            {
                used_fs! {
                    let offset = swaped_map.get(&start).unwrap();
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
        memory_map.insert(start + vma_offset, page_info);
    }

    used_fs! {
        let mut opt_buffer = Vec::new();
        for (&start, &off_in_swap) in swaped_map.range(start..end) {
            opt_buffer.push((start, off_in_swap));
        }
        for (start, off_in_swap) in opt_buffer {
            // opt for the swapped file, should copy swaped page for the new page.
            if !copy {
                swaped_map.remove(&start);
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
            swaped_map.insert(start + vma_offset, off_in_swap);
        }
    }
}

/// Allocate a section of physical memory for faulty pages
/// Since there is only one page table in RuxOS, the return value is the starting value
/// of a virtual address that is also mapped to the allocated physical address.
#[cfg(feature = "fs")]
pub(crate) fn preload_page_with_swap(
    memory_map: &mut BTreeMap<usize, PageInfo>,
    swaped_map: &mut BTreeMap<usize, Offset>,
    off_pool: &mut Vec<usize>,
) -> VirtAddr {
    match alloc_page_preload() {
        Ok(vaddr) => vaddr,
        // Try to swap the mapped memory into Disk and use this segment of physical memory
        #[cfg(feature = "fs")]
        Err(PagingError::NoMemory) => match memory_map.pop_first() {
            // For file mapping, the mapped content will be written directly to the original file.
            Some((vaddr_swapped, Some((fid, offset, size)))) => {
                let offset = offset.try_into().unwrap();
                sys_pwrite64(fid, vaddr_swapped as *mut c_void, size, offset);
                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
            }
            // For anonymous mapping, you need to save the mapped memory to the prepared swap file,
            //  and record the memory address and its offset in the swap file.
            Some((vaddr_swapped, None)) => {
                let offset_get = off_pool.pop();
                if SWAP_FID.is_negative() || offset_get.is_none() {
                    panic!(
                        "No free memory for mmap or swap fid, swap fid={}",
                        *SWAP_FID
                    );
                }
                let offset = offset_get.unwrap();
                swaped_map.insert(vaddr_swapped, offset);

                sys_pwrite64(
                    *SWAP_FID,
                    vaddr_swapped as *mut c_void,
                    PAGE_SIZE_4K,
                    offset as i64,
                );

                pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
            }
            _ => panic!("No memory for mmap, check if huge memory leaky exists"),
        },

        Err(ecode) => panic!(
            "Unexpected error {:x?} happening when page fault occurs!",
            ecode
        ),
    }
}
