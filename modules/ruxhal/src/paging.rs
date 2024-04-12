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
use crate::arch::flush_tlb;
use spinlock::SpinNoIrq;

use crate::mem::{
    direct_virt_to_phys, memory_regions, phys_to_virt, MemRegionFlags, PhysAddr, VirtAddr,
    PAGE_SIZE_4K,
};
use axalloc::global_allocator;
use lazy_init::LazyInit;

#[doc(no_inline)]
use page_table::{MappingFlags, PageSize, PagingError, PagingIf, PagingResult};

impl From<MemRegionFlags> for MappingFlags {
    fn from(f: MemRegionFlags) -> Self {
        let mut ret = Self::empty();
        if f.contains(MemRegionFlags::READ) {
            ret |= Self::READ;
        }
        if f.contains(MemRegionFlags::WRITE) {
            ret |= Self::WRITE;
        }
        if f.contains(MemRegionFlags::EXECUTE) {
            ret |= Self::EXECUTE;
        }
        if f.contains(MemRegionFlags::DEVICE) {
            ret |= Self::DEVICE;
        }
        if f.contains(MemRegionFlags::UNCACHED) {
            ret |= Self::UNCACHED;
        }
        ret
    }
}

/// Implementation of [`PagingIf`], to provide physical memory manipulation to
/// the [page_table] crate.
pub struct PagingIfImpl;

impl PagingIf for PagingIfImpl {
    fn alloc_frame() -> Option<PhysAddr> {
        global_allocator()
            .alloc_pages(1, PAGE_SIZE_4K)
            .map(|vaddr| direct_virt_to_phys(vaddr.into()))
            .ok()
    }

    fn dealloc_frame(paddr: PhysAddr) {
        global_allocator().dealloc_pages(phys_to_virt(paddr).as_usize(), 1)
    }

    #[inline]
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        /// The architecture-specific page table.
        pub type PageTable = page_table::x86_64::X64PageTable<PagingIfImpl>;
    } else if #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))] {
        /// The architecture-specific page table.
        pub type PageTable = page_table::riscv::Sv39PageTable<PagingIfImpl>;
    } else if #[cfg(target_arch = "aarch64")]{
        /// The architecture-specific page table.
        pub type PageTable = page_table::aarch64::A64PageTable<PagingIfImpl>;
    }
}

pub(crate) static KERNEL_PAGE_TABLE: LazyInit<SpinNoIrq<PageTable>> = LazyInit::new();

/// Remap the regions for kernel memory
pub fn remap_kernel_memory() -> PagingResult {
    if crate::cpu::this_cpu_is_bsp() {
        let mut kernel_page_table = PageTable::try_new()?;
        for r in memory_regions() {
            kernel_page_table.map_region(
                phys_to_virt(r.paddr),
                r.paddr,
                r.size,
                r.flags.into(),
                true,
            )?;
        }

        KERNEL_PAGE_TABLE.init_by(SpinNoIrq::new(kernel_page_table));
    }
    unsafe { crate::arch::write_page_table_root(KERNEL_PAGE_TABLE.lock().root_paddr()) };
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
    let mut kernel_page_table = KERNEL_PAGE_TABLE.lock();
    let (paddr, _) = kernel_page_table.unmap(swaped_vaddr)?;
    flush_tlb(Some(swaped_vaddr));
    Ok(phys_to_virt(paddr))
}

/// Map memory for an mmap-induced PageFault and updating PTE entries,
/// This function must be called after `alloc_page_preload` and
/// `pte_swap_preload` when the mapping operator is ready.
pub fn do_pte_map(vaddr: VirtAddr, fake_vaddr: VirtAddr, flags: MappingFlags) -> PagingResult {
    KERNEL_PAGE_TABLE.lock().map(
        vaddr,
        direct_virt_to_phys(fake_vaddr),
        PageSize::Size4K,
        flags,
    )
}

/// Query PTE entries of the virtual address.
///
/// get the physical address information corresponding to the virtual address from the page table
pub fn pte_query(vaddr: VirtAddr) -> PagingResult<(PhysAddr, MappingFlags, PageSize)> {
    let kernel_page_table = KERNEL_PAGE_TABLE.lock();
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
    KERNEL_PAGE_TABLE.lock().update(vaddr, paddr, flags)?;
    flush_tlb(Some(vaddr));
    Ok(())
}

/// Unmapping and decalloc memory for an page in page table.
///
/// release the corresponding memory at the same time
pub fn pte_unmap_page(vaddr: VirtAddr) -> PagingResult {
    trace!("unmapping vaddr: 0x{:x?}", vaddr);
    let (paddr, _) = KERNEL_PAGE_TABLE.lock().unmap(vaddr)?;
    global_allocator().dealloc_pages(phys_to_virt(paddr).as_usize(), 1);
    flush_tlb(Some(vaddr));
    Ok(())
}
