/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Physical memory management.

use core::fmt;

#[doc(no_inline)]
pub use memory_addr::{PhysAddr, VirtAddr, PAGE_SIZE_4K};

bitflags::bitflags! {
    /// The flags of a physical memory region.
    pub struct MemRegionFlags: usize {
        /// Readable.
        const READ          = 1 << 0;
        /// Writable.
        const WRITE         = 1 << 1;
        /// Executable.
        const EXECUTE       = 1 << 2;
        /// Device memory. (e.g., MMIO regions)
        const DEVICE        = 1 << 4;
        /// Uncachable memory. (e.g., framebuffer)
        const UNCACHED      = 1 << 5;
        /// Reserved memory, do not use for allocation.
        const RESERVED      = 1 << 6;
        /// Free memory for allocation.
        const FREE          = 1 << 7;
    }
}

impl fmt::Debug for MemRegionFlags {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.0, f)
    }
}

/// A physical memory region.
#[derive(Debug)]
pub struct MemRegion {
    /// The start physical address of the region.
    pub paddr: PhysAddr,
    /// The size in bytes of the region.
    pub size: usize,
    /// The region flags, see [`MemRegionFlags`].
    pub flags: MemRegionFlags,
    /// The region name, used for identification.
    pub name: &'static str,
}

/// A trait for address translation.
#[crate_interface::def_interface]
pub trait AddressTranslate {
    /// Translates a virtual address to a physical address.
    fn virt_to_phys(vaddr: VirtAddr) -> Option<usize> {
        Some(direct_virt_to_phys(vaddr).into())
    }
}

/// translates a virtual address to a physical address.
pub fn address_translate(vaddr: VirtAddr) -> Option<usize> {
    crate_interface::call_interface!(AddressTranslate::virt_to_phys, vaddr)
}

/// Converts a virtual address to a physical address.
///
/// It assumes that there is a linear mapping with the offset
/// [`PHYS_VIRT_OFFSET`], that maps all the physical memory to the virtual
/// space at the address plus the offset. So we have
/// `paddr = vaddr - PHYS_VIRT_OFFSET`.
/// Usually only used when the page table is not enabled or linear mapping
///
/// [`PHYS_VIRT_OFFSET`]: ruxconfig::PHYS_VIRT_OFFSET
#[inline]
pub const fn direct_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    PhysAddr::from(vaddr.as_usize() - ruxconfig::PHYS_VIRT_OFFSET)
}

/// Converts a physical address to a virtual address.
///
/// It assumes that there is a linear mapping with the offset
/// [`PHYS_VIRT_OFFSET`], that maps all the physical memory to the virtual
/// space at the address plus the offset. So we have
/// `vaddr = paddr + PHYS_VIRT_OFFSET`.
///
/// [`PHYS_VIRT_OFFSET`]: ruxconfig::PHYS_VIRT_OFFSET
#[inline]
pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    VirtAddr::from(paddr.as_usize() + ruxconfig::PHYS_VIRT_OFFSET)
}

/// Returns an iterator over all physical memory regions.
pub fn memory_regions() -> impl Iterator<Item = MemRegion> {
    kernel_image_regions().chain(crate::platform::mem::platform_regions())
}

/// Returns the memory regions of the kernel image (code and data sections).
fn kernel_image_regions() -> impl Iterator<Item = MemRegion> {
    [
        MemRegion {
            paddr: direct_virt_to_phys((_stext as usize).into()),
            size: _etext as usize - _stext as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::EXECUTE,
            name: ".text",
        },
        MemRegion {
            paddr: direct_virt_to_phys((_srodata as usize).into()),
            size: _erodata as usize - _srodata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ,
            name: ".rodata",
        },
        MemRegion {
            paddr: direct_virt_to_phys((_sdata as usize).into()),
            size: _edata as usize - _sdata as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: ".data .tdata .tbss .percpu",
        },
        MemRegion {
            paddr: direct_virt_to_phys((boot_stack as usize).into()),
            size: boot_stack_top as usize - boot_stack as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: "boot stack",
        },
        MemRegion {
            paddr: direct_virt_to_phys((_sbss as usize).into()),
            size: _ebss as usize - _sbss as usize,
            flags: MemRegionFlags::RESERVED | MemRegionFlags::READ | MemRegionFlags::WRITE,
            name: ".bss",
        },
    ]
    .into_iter()
}

/// Returns the default MMIO memory regions (from [`ruxconfig::MMIO_REGIONS`]).
#[allow(dead_code)]
pub(crate) fn default_mmio_regions() -> impl Iterator<Item = MemRegion> {
    ruxconfig::MMIO_REGIONS.iter().map(|reg| MemRegion {
        paddr: reg.0.into(),
        size: reg.1,
        flags: MemRegionFlags::RESERVED
            | MemRegionFlags::DEVICE
            | MemRegionFlags::READ
            | MemRegionFlags::WRITE,
        name: "mmio",
    })
}

/// Returns the default MMIO memory regions (from [`ruxconfig::DTB_ADDR`]).
#[allow(dead_code)]
pub(crate) fn default_dtb_regions() -> impl Iterator<Item = MemRegion> {
    let dtb_addr = PhysAddr::from(ruxconfig::DTB_ADDR).align_up_4k();
    let dtb_resgion = MemRegion {
        paddr: dtb_addr,
        size: 0x80000,
        flags: MemRegionFlags::READ | MemRegionFlags::WRITE | MemRegionFlags::EXECUTE,
        name: "dtb region",
    };
    core::iter::once(dtb_resgion)
}

/// Returns the default free memory regions (kernel image end to physical memory end).
#[allow(dead_code)]
pub(crate) fn default_free_regions() -> impl Iterator<Item = MemRegion> {
    let start_free = direct_virt_to_phys((_ekernel as usize).into()).align_up_4k();
    let end = PhysAddr::from(ruxconfig::PHYS_MEMORY_END).align_down_4k();

    let region_free = MemRegion {
        paddr: start_free,
        size: end.as_usize() - start_free.as_usize(),
        flags: MemRegionFlags::FREE
            | MemRegionFlags::READ
            | MemRegionFlags::WRITE
            | MemRegionFlags::EXECUTE,
        name: "free memory",
    };

    core::iter::once(region_free)
}

/// Fills the `.bss` section with zeros.
#[allow(dead_code)]
pub(crate) fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(_sbss as usize as *mut u8, _ebss as usize - _sbss as usize)
            .fill(0);
    }
}

extern "C" {
    fn _skernel();
    fn _stext();
    fn _etext();
    fn _srodata();
    fn _erodata();
    fn _sdata();
    fn _edata();
    fn _sbss();
    fn _ebss();
    fn _ekernel();
    fn boot_stack();
    fn boot_stack_top();
}
