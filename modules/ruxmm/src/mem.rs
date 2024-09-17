/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg(feature = "paging")]
use crate::paging::pte_query;

use ruxdriver::virtio::AddressTranslate;
use ruxhal::mem::{direct_virt_to_phys, PhysAddr, VirtAddr};

#[cfg(feature = "paging")]
struct AddressTranslateImpl;

/// Converts a virtual address to a physical address.
///
/// When paging is enabled, query physical address from the page table
#[cfg(feature = "paging")]
#[crate_interface::impl_interface]
impl AddressTranslate for AddressTranslateImpl {
    fn virt_to_phys(vaddr: VirtAddr) -> Option<usize> {
        match pte_query(vaddr) {
            Ok((paddr, _, _)) => Some(paddr.into()),
            Err(_) => None, // for address unmapped
        }
    }
}

/// Converts a virtual address to a physical address.
///
/// When paging is enabled, query physical address from the page table
#[cfg(not(feature = "paging"))]
#[crate_interface::impl_interface]
impl AddressTranslate for AddressTranslateImpl {
    fn virt_to_phys(vaddr: VirtAddr) -> Option<usize> {
        Some(direct_virt_to_phys(vaddr))
    }
}
