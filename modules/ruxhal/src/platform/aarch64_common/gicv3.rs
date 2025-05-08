/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::{irq::IrqHandler, mem::phys_to_virt};
use arm_gic::gic_v3::{GicDistributor, GicRedistributor};
use arm_gic::{translate_irq, InterruptType};
use lazy_init::LazyInit;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

/// The maximum number of IRQs.
pub const MAX_IRQ_COUNT: usize = 1024;

/// The virt timer IRQ number.
/// Physical timer IRQ number is 14
pub const TIMER_IRQ_NUM: usize = translate_irq(11, InterruptType::PPI).unwrap();

#[cfg(not(feature = "virtio_console"))]
/// The UART IRQ number.
pub const UART_IRQ_NUM: usize = translate_irq(ruxconfig::UART_IRQ, InterruptType::SPI).unwrap();

#[cfg(all(feature = "irq", feature = "virtio_console"))]
/// The Virtio-console IRQ number
pub const VIRTIO_CONSOLE_IRQ_NUM: usize =
    translate_irq(ruxconfig::VIRTIO_CONSOLE_IRQ, InterruptType::SPI).unwrap();

const GICD_BASE: PhysAddr = PhysAddr::from(ruxconfig::GICD_PADDR);
const GICR_BASE: PhysAddr = PhysAddr::from(ruxconfig::GICR_PADDR);
const GICR_STRIDE: usize = 0x20000;

static GICD: SpinNoIrq<GicDistributor> =
    SpinNoIrq::new(GicDistributor::new(phys_to_virt(GICD_BASE).as_mut_ptr()));

static GICR: LazyInit<SpinNoIrq<GicRedistributor>> = LazyInit::new();

/// Enables or disables the given IRQ.
pub fn set_enable(irq_num: usize, enabled: bool) {
    trace!("set enable: {} {}", irq_num, enabled);
    if irq_num < 32 {
        GICR.lock().set_enable(irq_num as _, enabled);
    } else {
        GICD.lock().set_enable(irq_num as _, enabled);
    }
}

/// Registers an IRQ handler for the given IRQ.
///
/// It also enables the IRQ if the registration succeeds. It returns `false` if
/// the registration failed.
pub fn register_handler(irq_num: usize, handler: IrqHandler) -> bool {
    trace!("register handler irq {}", irq_num);
    crate::irq::register_handler_common(irq_num, handler)
}

/// Dispatches the IRQ.
///
/// This function is called by the common interrupt handler. It looks
/// up in the IRQ handler table and calls the corresponding handler. If
/// necessary, it also acknowledges the interrupt controller after handling.
pub fn dispatch_irq(_unused: usize) {
    GICR.lock()
        .handle_irq(|irq_num| crate::irq::dispatch_irq_common(irq_num as _));
}

/// Initializes GICD, GICC on the primary CPU.
pub(crate) fn init_primary(cpu_id: usize) {
    info!("Initialize GICv3...");
    debug!("GICv3 GICR addr {:x}", GICR_BASE + cpu_id * GICR_STRIDE);
    GICR.init_by(SpinNoIrq::new(GicRedistributor::new(
        phys_to_virt(GICR_BASE + cpu_id * GICR_STRIDE).as_mut_ptr(),
    )));
    GICR.lock().init();
    GICD.lock().init();
}

/// Initializes GICC on secondary CPUs.
#[cfg(feature = "smp")]
pub(crate) fn init_secondary() {}
