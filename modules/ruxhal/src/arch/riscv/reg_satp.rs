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
