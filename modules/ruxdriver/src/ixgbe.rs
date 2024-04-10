/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axalloc::global_allocator;
use core::{alloc::Layout, ptr::NonNull};
use driver_net::ixgbe::{IxgbeHal, PhysAddr as IxgbePhysAddr};
use ruxhal::mem::{direct_virt_to_phys, phys_to_virt};

pub struct IxgbeHalImpl;

unsafe impl IxgbeHal for IxgbeHalImpl {
    fn dma_alloc(size: usize) -> (IxgbePhysAddr, NonNull<u8>) {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let vaddr = if let Ok(vaddr) = global_allocator().alloc(layout) {
            vaddr
        } else {
            return (0, NonNull::dangling());
        };
        let paddr = direct_virt_to_phys((vaddr.as_ptr() as usize).into());
        (paddr.as_usize(), vaddr)
    }

    unsafe fn dma_dealloc(_paddr: IxgbePhysAddr, vaddr: NonNull<u8>, size: usize) -> i32 {
        let layout = Layout::from_size_align(size, 8).unwrap();
        global_allocator().dealloc(vaddr, layout);
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: IxgbePhysAddr, _size: usize) -> NonNull<u8> {
        NonNull::new(phys_to_virt(paddr.into()).as_mut_ptr()).unwrap()
    }

    unsafe fn mmio_virt_to_phys(vaddr: NonNull<u8>, _size: usize) -> IxgbePhysAddr {
        direct_virt_to_phys((vaddr.as_ptr() as usize).into()).into()
    }

    fn wait_until(duration: core::time::Duration) -> Result<(), &'static str> {
        ruxhal::time::busy_wait_until(duration);
        Ok(())
    }
}
