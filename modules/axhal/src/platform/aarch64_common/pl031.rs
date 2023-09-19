use core::intrinsics::{volatile_load, volatile_store};
use core::fmt;

static RTC_DR: u32 = 0x000;
static RTC_MR: u32 = 0x004;
static RTC_LR: u32 = 0x008;
static RTC_CR: u32 = 0x00c;
static RTC_IMSC: u32 = 0x010;
static RTC_RIS: u32 = 0x014;
static RTC_MIS: u32 = 0x018;
static RTC_ICR: u32 = 0x01c;

pub static mut PL031_RTC: Pl031rtc = Pl031rtc {
    address: 0,
};

pub fn init() {
    info!("pl031 init begin");
    unsafe{
        PL031_RTC.init();
        let x = rtc_read_time();
        debug!("{}",x);
        let x = rtc_read_time();
        debug!("{}",x);
    }
}

pub struct Pl031rtc {
    pub address: usize,
}

pub const PHYS_RTC: usize = axconfig::PHYS_VIRT_OFFSET + 0x09010000;

impl fmt::Display for Pl031rtc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f,"RTC DR: {}\n",unsafe { self.read(RTC_DR) } as u64)?;
        writeln!(f,"RTC MR: {}\n",unsafe { self.read(RTC_MR) } as u64)?;
        writeln!(f,"RTC LR: {}\n",unsafe { self.read(RTC_LR) } as u64)?;
        writeln!(f,"RTC CR: {}\n",unsafe { self.read(RTC_CR) } as u64)?;
        writeln!(f,"RTC_IMSC: {}\n",unsafe { self.read(RTC_IMSC) } as u64)
    }
}

impl Pl031rtc {
    fn debug(&mut self) {
        use axlog::ax_println;
        ax_println!("{}",self);
    }

    fn init(&mut self) {
        self.address = PHYS_RTC;
        unsafe{
            if self.read(RTC_CR) != 1 {
                self.write(RTC_CR,1);
            }
        }
        self.debug();
    }

    pub unsafe fn read(&self, reg: u32) -> u32 {
        volatile_load((PHYS_RTC + reg as usize) as *const u32)
    }

    pub unsafe fn write(&mut self, reg: u32, value: u32) {
        volatile_store((PHYS_RTC + reg as usize) as *mut u32, value);
        self.debug();
    }

    pub fn time(&mut self) -> u64 {
        (unsafe { self.read(RTC_DR) } as u64)
    }
}

pub fn rtc_read_time() -> u64{
    unsafe {
        PL031_RTC.time()
    }
}

pub fn rtc_write_time(seconds:u32){
    unsafe { 
        PL031_RTC.write(RTC_LR,seconds) 
    };
}