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
    #[cfg(not(feature = "gic-v3"))]
    pub use crate::platform::aarch64_common::gicv2::*;
    #[cfg(feature = "gic-v3")]
    pub use crate::platform::aarch64_common::gicv3::*;
}

pub mod console {
    #[cfg(not(feature = "virtio_console"))]
    pub use crate::platform::aarch64_common::pl011::*;
    #[cfg(feature = "virtio_console")]
    pub use crate::virtio::virtio_console::*;
}

pub mod time {
    pub use crate::platform::aarch64_common::generic_timer::*;
    #[cfg(feature = "rtc")]
    pub use crate::platform::aarch64_common::pl031::*;
}

pub mod misc {
    pub use crate::platform::aarch64_common::psci::system_off as terminate;
}

extern "C" {
    fn exception_vector_base();
    fn rust_main(cpu_id: usize, dtb: usize);
    #[cfg(feature = "smp")]
    fn rust_main_secondary(cpu_id: usize);
}

pub(crate) unsafe extern "C" fn rust_entry(cpu_id: usize, dtb: usize) {
    crate::mem::clear_bss();
    crate::arch::set_exception_vector_base(exception_vector_base as usize);
    crate::arch::write_page_table_root0(0.into()); // disable low address access
    unsafe {
        dtb::init(crate::mem::phys_to_virt(dtb.into()).as_ptr());
    }
    crate::cpu::init_primary(cpu_id);
    #[cfg(not(feature = "virtio_console"))]
    super::aarch64_common::pl011::init_early();
    super::aarch64_common::generic_timer::init_early();
    rust_main(cpu_id, dtb);
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
pub fn platform_init(cpu_id: usize) {
    #[cfg(feature = "irq")]
    #[cfg(not(feature = "gic-v3"))]
    super::aarch64_common::gicv2::init_primary(cpu_id);
    #[cfg(feature = "irq")]
    #[cfg(feature = "gic-v3")]
    super::aarch64_common::gicv3::init_primary(cpu_id);
    super::aarch64_common::generic_timer::init_percpu();
    #[cfg(feature = "rtc")]
    super::aarch64_common::pl031::init();
    #[cfg(not(feature = "virtio_console"))]
    super::aarch64_common::pl011::init();
    #[cfg(feature = "virtio_console")]
    crate::virtio::virtio_console::enable_interrupt();
}

/// Initializes the platform devices for secondary CPUs.
#[cfg(feature = "smp")]
pub fn platform_init_secondary() {
    #[cfg(feature = "irq")]
    #[cfg(not(feature = "gic-v3"))]
    super::aarch64_common::gicv2::init_secondary();
    #[cfg(feature = "irq")]
    #[cfg(feature = "gic-v3")]
    super::aarch64_common::gicv3::init_secondary();
    super::aarch64_common::generic_timer::init_percpu();
}
