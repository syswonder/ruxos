/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
pub mod mem;

#[cfg(feature = "smp")]
pub mod mp;

#[cfg(feature = "irq")]
pub mod irq {
    pub use crate::platform::aarch64_common::gic::*;
}

pub mod console {
    pub use crate::platform::aarch64_common::pl011::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
    #[cfg(feature = "rtc")]
    pub use crate::platform::aarch64_common::pl031::*;
}

pub mod misc {
    pub fn terminate() -> ! {
        info!("Shutting down...");
        loop {
            crate::arch::halt();
        }
    }
}

extern "C" {
    fn exception_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize) {
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    unsafe {
        // Set the physical address of the dtb file to 0x03000000 in config.txt
        dtb::init(crate::mem::phys_to_virt(0x03000000.into()).as_ptr());
    }
    crate::cpu::init_primary(cpu_id);
    super::aarch64_common::pl011::init_early();
    super::aarch64_common::generic_timer::init_early();
    rust_main(cpu_id, 0x03000000);
}

#[cfg(feature = "smp")]
pub(crate) unsafe extern "C" fn rust_entry_secondary(cpu_id: usize) {
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    crate::cpu::init_secondary(cpu_id);
    rust_main_secondary(cpu_id);
}

/// Initializes the platform devices for the primary CPU.
///
/// For example, the interrupt controller and the timer.
pub fn platform_init() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_primary();
    super::aarch64_common::generic_timer::init_percpu();
    #[cfg(feature = "rtc")]
    super::aarch64_common::pl031::init();
    super::aarch64_common::pl011::init();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    super::aarch64_common::gic::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}
