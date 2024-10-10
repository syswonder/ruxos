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
use {alloc::sync::Arc, page_table::PagingError, ruxtask::fs::File};

use alloc::{collections::BTreeMap, vec::Vec};
use core::{
    cmp::{max, min},
    ops::{Bound, DerefMut},
};
use memory_addr::PAGE_SIZE_4K;
use page_table::MappingFlags;
use ruxhal::mem::VirtAddr;
use ruxmm::paging::{alloc_page_preload, do_pte_map, pte_query, pte_swap_preload, pte_unmap_page};
use ruxtask::vma::{FileInfo, PageInfo, SwapInfo, BITMAP_FREE, SWAPED_MAP, SWAP_FILE};
use ruxtask::{current, vma::Vma};

pub(crate) const VMA_START: usize = ruxconfig::MMAP_START_VADDR;
pub(crate) const VMA_END: usize = ruxconfig::MMAP_END_VADDR;

// use `used_fs` instead of `#[cfg(feature = "fs")]{}` to cancel the scope of code.
#[cfg(feature = "fs")]
macro_rules! used_fs {
     ($($code:tt)*) => {$($code)*};
 }

#[cfg(not(feature = "fs"))]
macro_rules! used_fs {
    ($($code:tt)*) => {};
}

/// read from target file
#[cfg(feature = "fs")]
pub(crate) fn read_from(file: &Arc<File>, buf: *mut u8, offset: u64, len: usize) {
    let src = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let actual_len = file
        .inner
        .read()
        .read_at(offset, src)
        .expect("read_from failed");
    if len != actual_len {
        warn!("read_from len=0x{len:x} but actual_len=0x{actual_len:x}");
    }
}

/// write into target file
#[cfg(feature = "fs")]
pub(crate) fn write_into(file: &Arc<File>, buf: *mut u8, offset: u64, len: usize) {
    let src = unsafe { core::slice::from_raw_parts_mut(buf, len) };
    let actual_len = file
        .inner
        .write()
        .write_at(offset, src)
        .expect("write_into failed");
    if len != actual_len {
        warn!("write_into len=0x{len:x} but actual_len=0x{actual_len:x}");
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

/// Clear the memory of the specified area. return Some(start) if successful.
/// take care of AA-deadlock, this function should not be used after `MEM_MAP` is used.
pub(crate) fn snatch_fixed_region(
    vma_map: &mut BTreeMap<usize, Vma>,
    start: usize,
    len: usize,
) -> Option<usize> {
    let end = start + len;

    // Return None if the specified address can't be used
    if start < VMA_START || end > VMA_END {
        return None;
    }

    let mut post_append: Vec<(usize, Vma)> = Vec::new(); // vma should be insert.
    let mut post_remove: Vec<usize> = Vec::new(); // vma should be removed.

    let mut node = vma_map.upper_bound_mut(Bound::Included(&start));
    while let Some(vma) = node.value_mut() {
        if vma.start_addr >= end {
            break;
        }
        if let Some((overlapped_start, overlapped_end)) =
            get_overlap((start, end), (vma.start_addr, vma.end_addr))
        {
            // add node for overlapped vma_ptr
            if vma.end_addr > overlapped_end {
                let right_vma = Vma::clone_from(vma, overlapped_end, vma.end_addr);
                post_append.push((overlapped_end, right_vma));
            }
            if overlapped_start > vma.start_addr {
                vma.end_addr = overlapped_start
            } else {
                post_remove.push(vma.start_addr);
            }
        }
        node.move_next();
    }

    // do action after success.
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

    Some(start)
}

/// release the range of [start, end) in mem_map
/// take care of AA-deadlock, this function should not be used after `MEM_MAP` is used.
pub(crate) fn release_pages_mapped(start: usize, end: usize) {
    let binding = current();
    let mut memory_map = binding.mm.mem_map.lock();
    let mut removing_vaddr = Vec::new();
    for (&vaddr, page_info) in memory_map.range(start..end) {
        #[cfg(feature = "fs")]
        if let Some(FileInfo { file, offset, size }) = &page_info.mapping_file {
            let src = vaddr as *mut u8;
            write_into(&file, src, *offset as u64, *size);
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

    let mut removing_vaddr = Vec::new();
    for (&vaddr, _) in swap_map.range(start..end) {
        removing_vaddr.push(vaddr);
    }
    for vaddr in removing_vaddr {
        swap_map.remove(&vaddr);
    }
}

/// shift mapped the page in both MEM_MAP and SWAPED_MAP.
/// No page fault here should be guaranteed
pub(crate) fn shift_mapped_page(start: usize, end: usize, vma_offset: usize, copy: bool) {
    let binding_task = current();
    let mut binding_mem_map = binding_task.mm.mem_map.lock();
    let memory_map = binding_mem_map.deref_mut();
    used_fs! {
        let mut swaped_map = SWAPED_MAP.lock();
        let mut off_pool = BITMAP_FREE.lock();
    }

    let mut opt_buffer = Vec::new();
    for (&start, page_info) in memory_map.range(start..end) {
        opt_buffer.push((start, page_info.clone()));
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
            let fake_vaddr = preload_page_with_swap(memory_map, &mut swaped_map, &mut off_pool);

            let dst = unsafe {
                core::slice::from_raw_parts_mut(fake_vaddr.as_usize() as *mut u8, PAGE_SIZE_4K)
            };
            if memory_map.contains_key(&start) {
                // copy the memory directly
                let src =
                    unsafe { core::slice::from_raw_parts_mut(start as *mut u8, PAGE_SIZE_4K) };
                dst.clone_from_slice(src);
            } else if page_info.mapping_file.is_none()
            /* has been swapped from memory */
            {
                used_fs! {
                    let swap_info = swaped_map.get(&start).unwrap();
                    read_from(&SWAP_FILE, start as *mut u8, swap_info.offset as u64, PAGE_SIZE_4K);
                }
            }
            (fake_vaddr, flags)
        };
        do_pte_map(VirtAddr::from(start + vma_offset), fake_vaddr, flags).unwrap();
        memory_map.insert(start + vma_offset, page_info.clone());
    }

    used_fs! {
        let mut opt_buffer = Vec::new();
        for (&start, &ref off_in_swap) in swaped_map.range(start..end) {
            opt_buffer.push((start, off_in_swap.clone()));
        }
        for (start, swap_info) in opt_buffer {
            // opt for the swapped file, should copy swaped page for the new page.
            if !copy {
                swaped_map.remove(&start);
            } else {
                let off_ptr = off_pool
                    .pop()
                    .expect("There are no free space in swap-file!");
                let mut rw_buffer: [u8; PAGE_SIZE_4K] = [0_u8; PAGE_SIZE_4K];
                read_from(&SWAP_FILE, rw_buffer.as_mut_ptr(), swap_info.offset as u64, PAGE_SIZE_4K);
                write_into(&SWAP_FILE, rw_buffer.as_mut_ptr(), off_ptr as u64, PAGE_SIZE_4K);
            }
            swaped_map.insert(start + vma_offset, swap_info.clone());
        }
    }
}

/// Allocate a section of physical memory for faulty pages
/// Since there is only one page table in RuxOS, the return value is the starting value
/// of a virtual address that is also mapped to the allocated physical address.
#[cfg(feature = "fs")]
pub(crate) fn preload_page_with_swap(
    memory_map: &mut BTreeMap<usize, Arc<PageInfo>>,
    swaped_map: &mut BTreeMap<usize, Arc<SwapInfo>>,
    off_pool: &mut Vec<usize>,
) -> VirtAddr {
    match alloc_page_preload() {
        Ok(vaddr) => vaddr,
        // Try to swap the mapped memory into Disk and use this segment of physical memory
        #[cfg(feature = "fs")]
        Err(PagingError::NoMemory) => match memory_map.pop_first() {
            // Some((vaddr_swapped, PageInfo{paddr:_, mapping_file:Some(FileInfo{file, offset, size})})) => {
            Some((vaddr_swapped, page_info)) => {
                match &page_info.mapping_file {
                    // For file mapping, the mapped content will be written directly to the original file.
                    Some(FileInfo { file, offset, size }) => {
                        let offset = *offset as u64;
                        write_into(&file, vaddr_swapped as *mut u8, offset, *size);
                        pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                    }
                    // For anonymous mapping, you need to save the mapped memory to the prepared swap file,
                    //  and record the memory address and its offset in the swap file.
                    None => {
                        let offset_get = off_pool.pop();
                        let offset = offset_get.unwrap();
                        swaped_map.insert(vaddr_swapped, Arc::new(offset.into()));

                        write_into(
                            &SWAP_FILE,
                            vaddr_swapped as *mut u8,
                            offset as u64,
                            PAGE_SIZE_4K,
                        );
                        pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
                    }
                }
            }
            // For anonymous mapping, you need to save the mapped memory to the prepared swap file,
            //  and record the memory address and its offset in the swap file.
            // Some((vaddr_swapped, PageInfo{paddr:_, mapping_file:Some(FileInfo{file, offset, size})})) => {
            //     let offset_get = off_pool.pop();
            //     let offset = offset_get.unwrap();
            //     swaped_map.insert(vaddr_swapped, Arc::new(offset));

            //     write_into(
            //         &SWAP_FILE,
            //         vaddr_swapped as *mut u8,
            //         offset as u64,
            //         PAGE_SIZE_4K,
            //     );
            //     pte_swap_preload(VirtAddr::from(vaddr_swapped)).unwrap()
            // }
            _ => panic!("No memory for mmap, check if huge memory leaky exists"),
        },

        Err(ecode) => panic!(
            "Unexpected error 0x{:x?} happening when page fault occurs!",
            ecode
        ),
    }
}
