/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg(feature = "fs")]
use crate::{
    ctypes,
    imp::mmap::utils::{preload_page_with_swap, read_from},
};
#[cfg(not(feature = "fs"))]
use ruxmm::paging::alloc_page_preload;
#[cfg(feature = "fs")]
use ruxtask::vma::{BITMAP_FREE, SWAPED_MAP, SWAP_FILE};

use crate::imp::mmap::utils::get_mflags_from_usize;
use alloc::sync::Arc;
use core::{
    cmp::min,
    ops::{Bound, DerefMut}, sync::atomic::{fence, Ordering},
};
use memory_addr::PAGE_SIZE_4K;
use page_table::MappingFlags;
use ruxhal::{
    mem::{direct_virt_to_phys, VirtAddr},
    trap::PageFaultCause,
};
use ruxtask::{
    current,
    vma::{FileInfo, PageInfo},
};

use ruxmm::paging::{do_pte_map, pte_query, pte_update_page};

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl ruxhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_page_fault(vaddr: usize, cause: PageFaultCause) -> bool {
        // warn!("----->handle_page_fault: vaddr=0x{:x?}, cause={:?}", vaddr, cause);
        
        // debug!("handle_page_fault: vaddr=0x{:x?}, cause={:?}", vaddr, cause);
        let binding_task = current();
        let mut binding_mem_map = binding_task.mm.vma_map.lock();
        let vma_map = binding_mem_map.deref_mut();
        if let Some(vma) = vma_map.upper_bound(Bound::Included(&vaddr)).value() {
            // Check if page existing in the vma, go to panic if not.
            if vma.end_addr <= vaddr {
                error!(
                    "Page Fault not match: vaddr=0x{:x?}, cause={:?}",
                    vaddr, cause
                );
                return false;
            }

            let vaddr = VirtAddr::from(vaddr).align_down_4k().as_usize();
            let size = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
            let map_flag = get_mflags_from_usize(vma.prot);

            trace!(
                "Page Fault Happening, vaddr:0x{:x?}, casue:{:?}, map_flags:0x{:x?}",
                vaddr,
                cause,
                map_flag
            );

            // Check if the access meet the prot
            if !map_flag.contains(cause.into()) {
                error!(
                    "Page Fault: Access violation, vaddr:0x{:x?}, cause:{:?}",
                    vaddr, cause
                );
                return false;
            }

            let binding_task = current();
            let mut binding_mem_map = binding_task.mm.mem_map.lock();
            let memory_map = binding_mem_map.deref_mut();
            used_fs! {
                let mut swaped_map = SWAPED_MAP.lock();
                let mut off_pool = BITMAP_FREE.lock();
            }

            // In a multi-threaded situation, it is possible that multiple threads
            // simultaneously trigger a page miss interrupt on the same page,
            // resulting in the page being actually mapped and causing an `AlreadyMap`
            // error
            let query_result = pte_query(VirtAddr::from(vaddr));
            let mem_item = memory_map.get(&vaddr);
            let is_cow = if let Ok((_, mapping_flags, _)) = query_result {
                assert!(mem_item.is_some());
                // Check if:
                // 1. the page is mapped by another thread.
                if mapping_flags.contains(cause.into()) {
                    return true;
                }
                // 2. the page is in Copy-on-Write mode so that it's set in read-only mode;
                assert!(mapping_flags.contains(MappingFlags::READ));
                assert!(!mapping_flags.contains(MappingFlags::WRITE));
                let mem_arc = mem_item.unwrap();
                if Arc::strong_count(mem_arc).eq(&1) {
                    // the last owner of the page, we can safely map it.
                    pte_update_page(vaddr.into(), None, Some(map_flag))
                        .expect("failed to update page table entry");
                    return true;
                }
                true
            } else {
                // no page table entry found, it means the page is not mapped yet.
                false
            };

            // Due to the existence of only one page table in ruxos, in
            // order to prevent data race in multi-threaded environ-
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
            let fake_vaddr = preload_page_with_swap(memory_map, &mut swaped_map, &mut off_pool);

            // Fill target data to assigned physical addresses, from file or zero according to mapping type
            let dst: *mut u8 = fake_vaddr.as_mut_ptr();

            if !is_cow {
                // get here if the page is belong to current process
                #[cfg(feature = "fs")]
                {
                    if let Some(swap_info) = swaped_map.remove(&vaddr) {
                        read_from(&SWAP_FILE, dst, swap_info.offset as u64, size);
                    } else if let Some(file) = &vma.file {
                        let off = (vma.offset + (vaddr - vma.start_addr)) as u64;
                        read_from(file, dst, off, size);
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
                #[cfg(feature = "fs")]
                if (vma.prot & ctypes::PROT_WRITE != 0)
                    && (vma.flags & ctypes::MAP_PRIVATE == 0)
                    && (vma.file.is_some())
                {
                    let map_length = min(PAGE_SIZE_4K, vma.end_addr - vaddr);
                    let offset = vma.offset + (vaddr - vma.start_addr);
                    let file_info = FileInfo {
                        file: vma.file.as_ref().unwrap().clone(),
                        offset,
                        size: map_length,
                    };
                    let page_info = PageInfo {
                        paddr: direct_virt_to_phys(fake_vaddr),
                        mapping_file: Some(file_info),
                    };
                    memory_map.insert(vaddr, Arc::new(page_info));
                } else {
                    memory_map.insert(
                        vaddr,
                        Arc::new(PageInfo {
                            paddr: direct_virt_to_phys(fake_vaddr),
                            mapping_file: None,
                        }),
                    );
                }
                #[cfg(not(feature = "fs"))]
                memory_map.insert(
                    vaddr,
                    Arc::new(PageInfo {
                        paddr: direct_virt_to_phys(VirtAddr::from(fake_vaddr)),
                    }),
                );

                // Do actual mmapping for target vaddr
                //
                // Note: other threads can access this page of memory after this code.
                match do_pte_map(VirtAddr::from(vaddr), fake_vaddr, map_flag) {
                    Ok(()) => true,
                    Err(_) => false,
                }
            } else {
                // get here if the page is belong to current process and is in Copy-on-Write mode.
                unsafe {
                    dst.copy_from(vaddr as *mut u8, size);
                }
                let paddr = direct_virt_to_phys(fake_vaddr);
                let mapping_file = memory_map
                    .get(&vaddr.into())
                    .unwrap()
                    .mapping_file
                    .clone();
                memory_map.remove(&vaddr.into());
                memory_map.insert(
                    vaddr.into(),
                    Arc::new(PageInfo {
                        paddr,
                        mapping_file,
                    }),
                );
                fence(Ordering::SeqCst);
                // Update the page table entry to map the physical address of the fake virtual address.
                match pte_update_page(vaddr.into(), Some(paddr), Some(map_flag)) {
                    Ok(()) => true,
                    Err(_) => false,
                }
            }
        } else {
            warn!("vaddr={:#x?},cause={:#x?}", vaddr, cause);
            false
        }
    }
}
