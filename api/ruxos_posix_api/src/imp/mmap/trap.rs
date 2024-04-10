/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;

#[cfg(not(feature = "fs"))]
use ruxhal::paging::alloc_page_preload;
#[cfg(feature = "fs")]
use {
    crate::imp::fs::sys_pread64,
    crate::imp::mmap::utils::{preload_page_with_swap, BITMAP_FREE, SWAPED_MAP, SWAP_FID},
};

use crate::imp::mmap::utils::{get_mflags_from_usize, MEM_MAP, VMA_MAP};
use core::{cmp::min, ffi::c_void, ops::Bound};
use memory_addr::PAGE_SIZE_4K;
use page_table::MappingFlags;
use ruxhal::{
    mem::VirtAddr,
    paging::{do_pte_map, pte_query},
    trap::PageFaultCause,
};

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl ruxhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_page_fault(vaddr: usize, cause: PageFaultCause) -> bool {
        let vma_map = VMA_MAP.lock();
        if let Some(vma) = vma_map.upper_bound(Bound::Included(&vaddr)).value() {
            // Check if page existing in the vma, go to panic if not.
            if vma.end_addr <= vaddr {
                return false;
            }

            let vaddr = VirtAddr::from(vaddr).align_down_4k().as_usize();
            let size = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
            let map_flag = get_mflags_from_usize(vma.prot);

            trace!(
                "Page Fault Happening, vaddr:{:x?}, casue:{:?}, map_flags:{:x?}",
                vaddr,
                cause,
                map_flag
            );

            // Check if the access meet the prot
            match cause {
                PageFaultCause::INSTRUCTION if !map_flag.contains(MappingFlags::EXECUTE) => {
                    return false
                }
                PageFaultCause::READ if !map_flag.contains(MappingFlags::READ) => return false,
                PageFaultCause::WRITE if !map_flag.contains(MappingFlags::WRITE) => return false,
                _ => {}
            }

            // In a multi-threaded situation, it is possible that multiple threads
            // simultaneously trigger a page miss interrupt on the same page,
            // resulting in the page being actually mapped and causing an `AlreadyMap`
            // error
            if pte_query(VirtAddr::from(vaddr)).is_ok() {
                return true;
            }

            let mut memory_map = MEM_MAP.lock();
            used_fs! {
                let mut swaped_map = SWAPED_MAP.lock();
                let mut off_pool = BITMAP_FREE.lock();
            }

            // Due to the existence of only one page table in ruxos, in
            // order to prevent data competition in multi-threaded environ-
            // -ments caused by adding the current virtual address to the
            // page table, it is necessary to first map the physical address
            // that needs to be mapped to another virtual address, and then
            // perform operations such as filling the corresponding memory
            // data. After completing all operations involving memory read
            // and write, map the actual virtual addresses that need to be mapped.
            //
            // fake_vaddr = preload() => do_pte_map(vaddr... fake_vaddr ...)
            #[cfg(not(feature = "fs"))]
            let fake_vaddr = alloc_page_preload().expect("alloc memory for new page failed");
            #[cfg(feature = "fs")]
            let fake_vaddr =
                preload_page_with_swap(&mut memory_map, &mut swaped_map, &mut off_pool);

            // Fill target data to assigned physical addresses, from file or zero according to mapping type
            let dst = fake_vaddr.as_mut_ptr() as *mut c_void;
            #[cfg(feature = "fs")]
            {
                if let Some(off) = swaped_map.remove(&vaddr) {
                    off_pool.push(off);
                    sys_pread64(*SWAP_FID, dst, size, off as i64);
                } else if vma.fid > 0 && !map_flag.is_empty() {
                    let off = (vma.offset + (vaddr - vma.start_addr)) as i64;
                    sys_pread64(vma.fid, dst, size, off);
                } else {
                    // Set page to 0 for anonymous mapping
                    //
                    // Safe because the page memory is allocated here
                    // and the page fault exception has not exited.
                    unsafe {
                        dst.write_bytes(0, size);
                    }
                }
            }

            // Set page to 0 for anonymous mapping
            //
            // Safe because the page memory is allocated here
            // and the page fault exception has not exited.
            #[cfg(not(feature = "fs"))]
            unsafe {
                dst.write_bytes(0, size);
            }

            // Insert the record into `MEM_MAP` with write-back information(`None` if no need to write-back).
            if (vma.prot & ctypes::PROT_WRITE != 0)
                && (vma.flags & ctypes::MAP_PRIVATE == 0)
                && (vma.fid > 0)
            {
                let map_length = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
                let offset = vma.offset + (vaddr - vma.start_addr);
                memory_map.insert(vaddr, Some((vma.fid, offset, map_length)));
            } else {
                memory_map.insert(vaddr, None);
            }

            // Do actual mmapping for target vaddr
            //
            // Note: other threads can access this page of memory after this code.
            match do_pte_map(VirtAddr::from(vaddr), fake_vaddr, map_flag) {
                Ok(()) => true,
                Err(_) => false,
            }
        } else {
            for mapped in vma_map.iter() {
                warn!("{:x?}", mapped);
            }
            warn!("vaddr={:x?},cause={:x?}", vaddr, cause);
            false
        }
    }
}
