use core::cmp::PartialEq;
use core::ops::{BitAnd, BitOr, Not};
use core::arch::asm;
use core::marker::PhantomData;
use lazy_init::LazyInit;

pub trait Io {
    type Value: Copy + PartialEq + BitAnd<Output = Self::Value> + BitOr<Output = Self::Value> + Not<Output = Self::Value>;

    fn read(&self) -> Self::Value;
    fn write(&mut self, value: Self::Value);

    #[inline(always)]
    fn readf(&self, flags: Self::Value) -> bool  {
        (self.read() & flags) as Self::Value == flags
    }

    #[inline(always)]
    fn writef(&mut self, flags: Self::Value, value: bool) {
        let tmp: Self::Value = match value {
            true => self.read() | flags,
            false => self.read() & !flags,
        };
        self.write(tmp);
    }
}

pub struct ReadOnly<I> {
    inner: I
}

impl<I> ReadOnly<I> {
    pub const fn new(inner: I) -> ReadOnly<I> {
        ReadOnly {
            inner: inner
        }
    }
}

impl<I: Io> ReadOnly<I> {
    #[inline(always)]
    pub fn read(&self) -> I::Value {
        self.inner.read()
    }

    #[inline(always)]
    pub fn readf(&self, flags: I::Value) -> bool {
        self.inner.readf(flags)
    }
}

pub struct WriteOnly<I> {
    inner: I
}

impl<I> WriteOnly<I> {
    pub const fn new(inner: I) -> WriteOnly<I> {
        WriteOnly {
            inner: inner
        }
    }
}

impl<I: Io> WriteOnly<I> {
    #[inline(always)]
    pub fn write(&mut self, value: I::Value) {
        self.inner.write(value)
    }

    #[inline(always)]
    pub fn writef(&mut self, flags: I::Value, value: bool) {
        self.inner.writef(flags, value)
    }
}


/// Generic PIO
#[derive(Copy, Clone)]
pub struct Pio<T> {
    port: u16,
    value: PhantomData<T>,
}

impl<T> Pio<T> {
    /// Create a PIO from a given port
    pub const fn new(port: u16) -> Self {
        Pio::<T> {
            port,
            value: PhantomData,
        }
    }
}

/// Read/Write for byte PIO
impl Io for Pio<u8> {
    type Value = u8;

    /// Read
    #[inline(always)]
    fn read(&self) -> u8 {
        let value: u8;
        unsafe {
            asm!("in al, dx", in("dx") self.port, out("al") value, options(nostack, nomem, preserves_flags));
        }
        value
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u8) {
        unsafe {
            asm!("out dx, al", in("dx") self.port, in("al") value, options(nostack, nomem, preserves_flags));
        }
    }
}

/// Read/Write for word PIO
impl Io for Pio<u16> {
    type Value = u16;

    /// Read
    #[inline(always)]
    fn read(&self) -> u16 {
        let value: u16;
        unsafe {
            asm!("in ax, dx", in("dx") self.port, out("ax") value, options(nostack, nomem, preserves_flags));
        }
        value
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u16) {
        unsafe {
            asm!("out dx, ax", in("dx") self.port, in("ax") value, options(nostack, nomem, preserves_flags));
        }
    }
}

/// Read/Write for doubleword PIO
impl Io for Pio<u32> {
    type Value = u32;

    /// Read
    #[inline(always)]
    fn read(&self) -> u32 {
        let value: u32;
        unsafe {
            asm!("in eax, dx", in("dx") self.port, out("eax") value, options(nostack, nomem, preserves_flags));
        }
        value
    }

    /// Write
    #[inline(always)]
    fn write(&mut self, value: u32) {
        unsafe {
            asm!("out dx, eax", in("dx") self.port, in("eax") value, options(nostack, nomem, preserves_flags));
        }
    }
}

pub static mut X86_RTC: LazyInit<Rtc> = LazyInit::new();

pub fn init() {
    let mut rtc = Rtc::new();
}

fn cvt_bcd(value: usize) -> usize {
    (value & 0xF) + ((value / 16) * 10)
}

fn cvt_dec(value: usize) -> usize{
    ((value / 10) <<4 ) | (value % 10)
}

/// RTC
pub struct Rtc {
    addr: Pio<u8>,
    data: Pio<u8>,
    nmi: bool,
}

impl Rtc {
    /// Create new empty RTC
    pub fn new() -> Self {
        Rtc {
            addr: Pio::<u8>::new(0x70),
            data: Pio::<u8>::new(0x71),
            nmi: false,
        }
    }

    /// Read
    unsafe fn read(&mut self, reg: u8) -> u8 {
        if self.nmi {
            self.addr.write(reg & 0x7F);
        } else {
            self.addr.write(reg | 0x80);
        }
        self.data.read()
    }

    /// Write
    #[allow(dead_code)]
    unsafe fn write(&mut self, reg: u8, value: u8) {
        if self.nmi {
            self.addr.write(reg & 0x7F);
        } else {
            self.addr.write(reg | 0x80);
        }
        self.data.write(value);
    }

    /// Wait for an update, can take one second if full is specified!
    unsafe fn wait(&mut self, full: bool) {
        if full {
            while self.read(0xA) & 0x80 != 0x80 {}
        }
        while self.read(0xA) & 0x80 == 0x80 {}
    }

    /// Get time without waiting
    pub unsafe fn time_no_wait(&mut self) -> u64 {
        /*let century_register = if let Some(ref fadt) = acpi::ACPI_TABLE.lock().fadt {
            Some(fadt.century)
        } else {
            None
        };*/

        let mut second = self.read(0) as usize;
        let mut minute = self.read(2) as usize;
        let mut hour = self.read(4) as usize;
        let mut day = self.read(7) as usize;
        let mut month = self.read(8) as usize;
        let mut year = self.read(9) as usize;
        let mut century = /* TODO: Fix invalid value from VirtualBox
        if let Some(century_reg) = century_register {
            self.read(century_reg) as usize
        } else */ {
            20
        };
        let register_b = self.read(0xB);

        if register_b & 4 != 4 {
            second = cvt_bcd(second);
            minute = cvt_bcd(minute);
            hour = cvt_bcd(hour & 0x7F) | (hour & 0x80);
            day = cvt_bcd(day);
            month = cvt_bcd(month);
            year = cvt_bcd(year);
            century = /* TODO: Fix invalid value from VirtualBox
            if century_register.is_some() {
                cvt_bcd(century)
            } else */ {
                century
            };
        }

        if register_b & 2 != 2 || hour & 0x80 == 0x80 {
            hour = ((hour & 0x7F) + 12) % 24;
        }

        year += century * 100;

        // Unix time from clock
        let mut secs: u64 = (year as u64 - 1970) * 31_536_000;

        let mut leap_days = (year as u64 - 1972) / 4 + 1;
        if year % 4 == 0 && month <= 2 {
            leap_days -= 1;
        }
        secs += leap_days * 86_400;

        match month {
            2 => secs += 2_678_400,
            3 => secs += 5_097_600,
            4 => secs += 7_776_000,
            5 => secs += 10_368_000,
            6 => secs += 13_046_400,
            7 => secs += 15_638_400,
            8 => secs += 18_316_800,
            9 => secs += 20_995_200,
            10 => secs += 23_587_200,
            11 => secs += 26_265_600,
            12 => secs += 28_857_600,
            _ => (),
        }

        secs += (day as u64 - 1) * 86_400;
        secs += hour as u64 * 3600;
        secs += minute as u64 * 60;
        secs += second as u64;

        secs
    }


    /// Get time
    pub fn time(&mut self) -> u64 {
        loop {
            unsafe {
                self.wait(false);
                let time = self.time_no_wait();
                self.wait(false);
                let next_time = self.time_no_wait();
                if time == next_time {
                    return time;
                }
            }
        }
    }

    pub unsafe fn write_time_no_wait(&mut self,unix_time:u32){
        let mut second = self.read(0) as usize;
        let mut minute = self.read(2) as usize;
        let mut hour = self.read(4) as usize;
        let mut day = self.read(7) as usize;
        let mut month = self.read(8) as usize;
        let mut year = self.read(9) as usize;
        let mut century = /* TODO: Fix invalid value from VirtualBox
        if let Some(century_reg) = century_register {
            self.read(century_reg) as usize
        } else */ {
            20
        };
        let register_b = self.read(0xB);


        debug!("{}, {}, {}, {}, {}, {}, {}, {}",second,minute,hour,day,month,year,register_b,register_b & 4);

        let secs = unix_time;
        let nsecs = 0;

        // 计算日期和时间
        let mut t = secs;
        let mut tdiv = t / 86400;
        let mut tt = t % 86400;
        let mut hour = tt / 3600;
        tt %= 3600;
        let mut min = tt / 60;
        tt %= 60;
        let mut sec = tt;
        let mut year = 1970;
        let mut mon = 1;
        let mut mday = 0;

        // 计算年、月和日
        while tdiv >= 365 {
            let days = if is_leap_year(year) { 366 } else { 365 };
            if tdiv >= days {
                tdiv -= days;
                year += 1;
            } else {
                break;
            }
        }

        while tdiv > 0 {
            let days = days_in_month(mon, year);
            if u64::from(tdiv) >= days {
                tdiv -= days as u32;
                mon += 1;
            } else {
                break;
            }
        }

        mday = tdiv + 1;

        year -= 2000;

        if register_b & 4 != 4 {
            sec = cvt_dec(sec as usize) as u32;
            min = cvt_dec(min as usize) as u32;
            mday = cvt_dec(mday as usize) as u32;
            mon = cvt_dec(mon as usize) as u64;
            year = cvt_dec(year as usize) as u64;
        }
        let mut bcd_value = hour % 10;
        let tens = hour / 10;
        if hour >= 12 {
            bcd_value |= 0x80;
        }
        bcd_value |= tens << 4;
        hour = bcd_value;

        debug!("{}, {}, {}, {}, {}, {}",sec,min,hour,mday,mon,year);
        self.write(0, sec as u8);
        self.write(2, min as u8);
        self.write(4, hour as u8);
        self.write(7, mday as u8);
        self.write(8, mon as u8);
        self.write(9, year as u8);
    }
}

fn is_leap_year(year: u64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

fn days_in_month(month: u64, year: u64) -> u64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}


pub fn rtc_read_time() -> u64{
    unsafe{
        if !X86_RTC.is_init(){
            X86_RTC.init_by(Rtc::new());
        }
        let mut rtc: &mut Rtc = X86_RTC.get_mut_unchecked();
        /*let mut rtc = Rtc::new();*/
        rtc.time()
    }
}

pub fn rtc_write_time(seconds:u32){
    unsafe{
        if !X86_RTC.is_init(){
            X86_RTC.init_by(Rtc::new());
        }
        let mut rtc: &mut Rtc = X86_RTC.get_mut_unchecked();
        /*let mut rtc = Rtc::new();*/
        rtc.write_time_no_wait(seconds);
    }
}