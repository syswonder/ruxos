/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use riscv::register::satp;

type PhysAddr = usize;

#[allow(clippy::upper_case_acronyms)]
pub struct PPN(usize);

impl From<PhysAddr> for PPN {
    fn from(value: PhysAddr) -> Self {
        let num = value;
        let page_frame_num = num >> 12;
        Self(page_frame_num)
    }
}

impl From<PPN> for usize {
    fn from(value: PPN) -> Self {
        value.0
    }
}
pub struct AddressSpaceID(usize);

impl From<u16> for AddressSpaceID {
    fn from(value: u16) -> Self {
        let value = usize::from(value);
        AddressSpaceID(value << 44)
    }
}

impl From<AddressSpaceID> for usize {
    fn from(value: AddressSpaceID) -> Self {
        value.0
    }
}

pub struct RegSatp;

impl RegSatp {
    pub fn gen_satp(mode: satp::Mode, asid: u16, page_table_addr: PhysAddr) -> usize {
        let mode = (mode as usize) << 60;
        let asid: AddressSpaceID = asid.into();
        let physical_page_num: PPN = page_table_addr.into();
        mode | usize::from(asid) | usize::from(physical_page_num)
    }
}
