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

use crate::mem::{
    direct_virt_to_phys, phys_to_virt, MemRegionFlags, PhysAddr, VirtAddr, PAGE_SIZE_4K,
};
use axalloc::global_allocator;

#[doc(no_inline)]
use page_table::{MappingFlags, PagingIf};
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
