/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

mod apic;
mod boot;
mod dtables;
mod uart16550;

pub mod mem;
pub mod misc;
#[cfg(feature = "rtc")]
pub mod rtc;
pub mod time;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use super::apic::*;
}

pub mod console {
    pub use super::uart16550::*;
}

extern "C" {
    fn rust_main(cpu_id: usize, dtb: usize) -> !;
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize) -> !;
}

fn current_cpu_id() -> usize {
    match raw_cpuid::CpuId::new().get_feature_info() {
        Some(finfo) => finfo.initial_local_apic_id() as usize,
        None => 0,
    }
}

use crate::COMLINE_BUF;
// find cmdline in multiboot info and save it in COMLINE_BUF
unsafe fn parse_cmdline(mbi: usize) {
    let mbi = mbi as *const u32;
    let flag = mbi.read();
    if (flag & (1 << 2)) > 0 {
        let cmdline = *mbi.add(4) as *const u8;
        let mut len = 0;
        while cmdline.add(len).read() != 0 {
            COMLINE_BUF[len] = cmdline.add(len).read();
            len += 1;
        }
    }
}

unsafe extern "C" fn rust_entry(magic: usize, mbi: usize) {
    // TODO: handle multiboot info
    if magic == self::boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::mem::clear_bss();
        crate::cpu::init_primary(current_cpu_id());
        self::uart16550::init();
        self::dtables::init_primary();
        #[cfg(feature = "musl")]
        crate::arch::init_syscall_entry();
        self::time::init_early();
        parse_cmdline(mbi);
        rust_main(current_cpu_id(), 0);
    }
}

#[allow(unused_variables)]
unsafe extern "C" fn rust_entry_secondary(magic: usize) {
    #[cfg(feature = "smp")]
    if magic == self::boot::MULTIBOOT_BOOTLOADER_MAGIC {
        crate::cpu::init_secondary(current_cpu_id());
        self::dtables::init_secondary();
        #[cfg(feature = "musl")]
        crate::arch::init_syscall_entry();
        rust_main_secondary(current_cpu_id());
    }
}

/// Initializes the platform devices for the primary CPU.
pub fn platform_init(_cpu_id: usize) {
    self::apic::init_primary();
    self::time::init_primary();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    self::apic::init_secondary();
    self::time::init_secondary();
}
