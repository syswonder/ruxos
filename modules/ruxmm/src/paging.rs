/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Page table manipulation.
extern crate alloc;

use ruxhal::arch::flush_tlb;
use ruxhal::mem::{
    direct_virt_to_phys, memory_regions, phys_to_virt, PhysAddr, VirtAddr, PAGE_SIZE_4K,
};

use axalloc::global_allocator;
#[doc(no_inline)]
use page_table::{MappingFlags, PageSize, PagingError, PagingResult};

use log::{trace, warn};

/// Remap the regions for kernel memory
pub fn remap_kernel_memory() -> PagingResult {
    let current_task = ruxtask::current();
    let mut kernel_page_table = current_task.pagetable.lock();
    if ruxhal::cpu::this_cpu_is_bsp() {
        for r in memory_regions() {
            kernel_page_table.map_region(
                phys_to_virt(r.paddr),
                r.paddr,
                r.size,
                r.flags.into(),
                true,
            )?;
        }
    }

    unsafe { ruxhal::arch::write_page_table_root(kernel_page_table.root_paddr()) };
    Ok(())
}

/// Temporarily, `malloc` alloc memory in heap simply, and it can not be swapped
/// into swap file. Once the memory is not enough with all memory alloced, it
/// will be too late, as there will be no memory for `malloc` any more. In practice,
/// this is highly likely to cause errors of insufficient memory. To prevent this,
/// mmapping will not alloc from physical address to avoid this.
///
/// After the page of `malloc` can be swapped, or it raises a propriately handler
/// to swap page when memory is not enough, it will be okay to delete this.
const PAGE_NUM_MIN: usize = 1024;

/// Obtain fake VirtAddr addresses without performing virtual memory mapping
/// to prevent physical competition between multiple threads.
/// After call the function. the page is alloced in allocator but its virtual
/// address is still on linear mapping region.
/// use `do_pte_map` to do actually page mapping after call this function.
pub fn alloc_page_preload() -> Result<VirtAddr, PagingError> {
    if global_allocator().available_pages() < PAGE_NUM_MIN {
        warn!(
            "available page num is {:?}",
            global_allocator().available_pages()
        );
        return Err(PagingError::NoMemory);
    };
    match global_allocator().alloc_pages(1, PAGE_SIZE_4K) {
        Ok(fake_vaddr) => Ok(VirtAddr::from(fake_vaddr)),
        Err(_) => Err(PagingError::NoMemory),
    }
}

/// Unmap memory for an mmap-induced PageFault and updating PTE entries.
/// After call the function. the page is alloced in allocator but its virtual
/// address is still on linear mapping region.
/// use `do_pte_map` to do actually page mapping after call this function.
pub fn pte_swap_preload(swaped_vaddr: VirtAddr) -> PagingResult<VirtAddr> {
    trace!("swapping swaped_vaddr: 0x{:x?}", swaped_vaddr,);
    let binding = ruxtask::current();
    let mut kernel_page_table = binding.pagetable.lock();
    let (paddr, _) = kernel_page_table.unmap(swaped_vaddr)?;
    flush_tlb(Some(swaped_vaddr));
    Ok(phys_to_virt(paddr))
}

/// Map memory for an mmap-induced PageFault and updating PTE entries,
/// This function must be called after `alloc_page_preload` and
/// `pte_swap_preload` when the mapping operator is ready.
pub fn do_pte_map(vaddr: VirtAddr, fake_vaddr: VirtAddr, flags: MappingFlags) -> PagingResult {
    let ret = ruxtask::current().pagetable.lock().map(
        vaddr,
        direct_virt_to_phys(fake_vaddr),
        PageSize::Size4K,
        flags,
    );
    ret
}

/// Query PTE entries of the virtual address.
///
/// get the physical address information corresponding to the virtual address from the page table
pub fn pte_query(vaddr: VirtAddr) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
    let binding = ruxtask::current();
    let kernel_page_table = binding.pagetable.lock();
    kernel_page_table.query(vaddr)
}

/// Update flags or physical address for an PTE entries.
///
/// change the physical address or access permissions mapped by the virtual address
pub fn pte_update_page(
    vaddr: VirtAddr,
    paddr: Option<PhysAddr>,
    flags: Option<MappingFlags>,
) -> PagingResult {
    trace!(
        "updating vaddr:0x{:x?} paddr:0x{:x?} flags:0x{:x?}",
        vaddr,
        paddr,
        flags
    );
    ruxtask::current()
        .pagetable
        .lock()
        .update(vaddr, paddr, flags)?;
    flush_tlb(Some(vaddr));
    Ok(())
}

/// Unmapping and decalloc memory for an page in page table.
///
/// release the corresponding memory at the same time
pub fn pte_unmap_page(vaddr: VirtAddr) -> PagingResult {
    trace!("unmapping vaddr: 0x{:x?}", vaddr);
    ruxtask::current().pagetable.lock().unmap(vaddr)?;
    flush_tlb(Some(vaddr));
    Ok(())
}
