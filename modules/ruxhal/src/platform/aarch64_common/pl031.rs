/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! PL031 RTC.

static RTC_DR: u32 = 0x000; //Data Register
static RTC_MR: u32 = 0x004; //Match Register
static RTC_LR: u32 = 0x008; //Load Register
static RTC_CR: u32 = 0x00c; //Control Register
static RTC_IMSC: u32 = 0x010; //Interrupt Mask Set or Clear register

const PHYS_RTC: usize = ruxconfig::PHYS_VIRT_OFFSET + 0x09010000;

static PL031_RTC: Pl031rtc = Pl031rtc { address: PHYS_RTC };

pub fn init() {
    info!("Initialize pl031 rtc...");
    PL031_RTC.init();
    debug!("{}", rtc_read_time());
}

struct Pl031rtc {
    address: usize,
}

impl core::fmt::Display for Pl031rtc {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "RTC DR: {}\nRTC MR: {}\nRTC LR: {}\nRTC CR: {}\nRTC_IMSC: {}",
            unsafe { self.read(RTC_DR) } as u64,
            unsafe { self.read(RTC_MR) } as u64,
            unsafe { self.read(RTC_LR) } as u64,
            unsafe { self.read(RTC_CR) } as u64,
            unsafe { self.read(RTC_IMSC) } as u64
        )
    }
}

impl Pl031rtc {
    fn init(&self) {
        unsafe {
            if self.read(RTC_CR) != 1 {
                self.write(RTC_CR, 1);
            }
        }
    }

    pub unsafe fn read(&self, reg: u32) -> u32 {
        core::ptr::read_volatile((self.address + reg as usize) as *const u32)
    }

    pub unsafe fn write(&self, reg: u32, value: u32) {
        core::ptr::write_volatile((self.address + reg as usize) as *mut u32, value);
    }

    pub fn time(&self) -> u64 {
        unsafe { self.read(RTC_DR) as u64 }
    }
}

pub fn rtc_read_time() -> u64 {
    PL031_RTC.time()
}

pub fn rtc_write_time(seconds: u32) {
    unsafe { PL031_RTC.write(RTC_LR, seconds) };
}
