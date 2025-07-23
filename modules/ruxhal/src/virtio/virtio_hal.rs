/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! virtio-hal implementation for RuxOS.
use crate::mem::{address_translate, direct_virt_to_phys, phys_to_virt};
use axalloc::global_allocator;
use core::ptr::NonNull;
use driver_virtio::VirtIoHal;
use virtio_drivers::{BufferDirection, PhysAddr};

/// virtio-hal struct define
pub struct VirtIoHalImpl;

/// virtio-hal trait implement
unsafe impl VirtIoHal for VirtIoHalImpl {
    /// Allocate DMA buffer
    fn dma_alloc(pages: usize, _direction: BufferDirection) -> (PhysAddr, NonNull<u8>) {
        let vaddr = if let Ok(vaddr) = global_allocator().alloc_pages(pages, 0x1000) {
            vaddr
        } else {
            return (0, NonNull::dangling());
        };
        let paddr = direct_virt_to_phys(vaddr.into());
        let ptr = NonNull::new(vaddr as _).unwrap();
        (paddr.as_usize(), ptr)
    }

    /// Deallocate DMA buffer
    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: NonNull<u8>, pages: usize) -> i32 {
        global_allocator().dealloc_pages(vaddr.as_ptr() as usize, pages);
        0
    }

    /// Convert physical address to virtual address
    #[inline]
    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    /// Share DMA buffer
    #[inline]
    unsafe fn share(buffer: NonNull<[u8]>, _direction: BufferDirection) -> PhysAddr {
        let vaddr = buffer.as_ptr() as *mut u8 as usize;
        address_translate(vaddr.into()).expect("virt_to_phys failed")
    }

    /// Unshare DMA buffer
    #[inline]
    unsafe fn unshare(_paddr: PhysAddr, _buffer: NonNull<[u8]>, _direction: BufferDirection) {}
}
