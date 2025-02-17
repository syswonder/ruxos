/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Types and definitions for GICv2.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0048/latest/>

use core::ptr::NonNull;

use crate::{TriggerMode, GIC_MAX_IRQ, SPI_RANGE, read_sysreg, write_sysreg};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

register_structs! {
    /// GIC Distributor registers.
    #[allow(non_snake_case)]
    GicDistributorRegs {
        /// Distributor Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Controller Type Register.
        (0x0004 => TYPER: ReadOnly<u32>),
        /// Distributor Implementer Identification Register.
        (0x0008 => IIDR: ReadOnly<u32>),
        (0x000c => _reserved_0),
        /// Interrupt Group Registers.
        (0x0080 => IGROUPR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Enable Registers.
        (0x0100 => ISENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Enable Registers.
        (0x0180 => ICENABLER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Pending Registers.
        (0x0200 => ISPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Pending Registers.
        (0x0280 => ICPENDR: [ReadWrite<u32>; 0x20]),
        /// Interrupt Set-Active Registers.
        (0x0300 => ISACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Clear-Active Registers.
        (0x0380 => ICACTIVER: [ReadWrite<u32>; 0x20]),
        /// Interrupt Priority Registers.
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Processor Targets Registers.
        (0x0800 => ITARGETSR: [ReadWrite<u32>; 0x100]),
        /// Interrupt Configuration Registers.
        (0x0c00 => ICFGR: [ReadWrite<u32>; 0x40]),
        (0x0d00 => _reserved_1),
        /// Non-secure Access Control Registers (GICD_NSACR)
        (0x0E00 => NSACR: [ReadWrite<u32>; 64]),
        (0x0F00 => _reserved_2),
        /// Interrupt Routing Registers (GICD_IROUTER)
        (0x6100 => IROUTER: [ReadWrite<u64>; 987]),
        (0x7FD8 => _reserved_3),
        /// Chip Status Register (GICD_CHIPSR)
        (0xC000 => CHIPSR: ReadWrite<u32>),
        /// Default Chip Register (GICD_DCHIPR)
        (0xC004 => DCHIPR: ReadWrite<u32>),
        /// Chip Registers (GICD_CHIPRn)
        (0xC008 => CHIPR: [ReadWrite<u64>; 0x10]),
        (0xC088 => _reserved_4),
        /// Interrupt Class Registers (GICD_ICLARn)
        (0xE008 => ICLAR: [ReadWrite<u32>; 0x40]),
        /// Interrupt Error Registers (GICD_IERRRn)
        (0xE108 => IERRR: [ReadWrite<u32>; 0x1e]),
        (0xE180 => _reserved_5),
        /// Configuration ID Register (GICD_CFGID)
        (0xF000 => CFGID: ReadOnly<u64>),
        (0xF008 => _reserved_6),
        /// Peripheral ID Registers
        (0xFFD0 => PIDR4: ReadOnly<u32>),
        (0xFFD4 => PIDR5: ReadOnly<u32>),
        (0xFFD8 => PIDR6: ReadOnly<u32>),
        (0xFFDC => PIDR7: ReadOnly<u32>),
        (0xFFE0 => PIDR0: ReadOnly<u32>),
        (0xFFE4 => PIDR1: ReadOnly<u32>),
        (0xFFE8 => PIDR2: ReadOnly<u32>),
        (0xFFEC => PIDR3: ReadOnly<u32>),
        /// Component ID Registers
        (0xFFF0 => CIDR0: ReadOnly<u32>),
        (0xFFF4 => CIDR1: ReadOnly<u32>),
        (0xFFF8 => CIDR2: ReadOnly<u32>),
        (0xFFFC => CIDR3: ReadOnly<u32>),
        (0x10000 => @END),
    }
}

register_structs! {
    /// GIC CPU Interface registers.
    #[allow(non_snake_case)]
    GicCpuInterfaceRegs {
        /// CPU Interface Control Register.
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Interrupt Priority Mask Register.
        (0x0004 => PMR: ReadWrite<u32>),
        /// Binary Point Register.
        (0x0008 => BPR: ReadWrite<u32>),
        /// Interrupt Acknowledge Register.
        (0x000c => IAR: ReadOnly<u32>),
        /// End of Interrupt Register.
        (0x0010 => EOIR: WriteOnly<u32>),
        /// Running Priority Register.
        (0x0014 => RPR: ReadOnly<u32>),
        /// Highest Priority Pending Interrupt Register.
        (0x0018 => HPPIR: ReadOnly<u32>),
        (0x001c => _reserved_1),
        /// CPU Interface Identification Register.
        (0x00fc => IIDR: ReadOnly<u32>),
        (0x0100 => _reserved_2),
        /// Deactivate Interrupt Register.
        (0x1000 => DIR: WriteOnly<u32>),
        (0x1004 => @END),
    }
}

register_structs! {
    /// GIC Redistributor Registers
    #[allow(non_snake_case)]
    GicRedistributorRegs {
        /// Redistributor Control Register (GICR_CTLR)
        (0x0000 => CTLR: ReadWrite<u32>),
        /// Redistributor Implementation Identification Register (GICR_IIDR)
        (0x0004 => IIDR: ReadOnly<u32>),
        /// Interrupt Controller Type Register (GICR_TYPER)
        (0x0008 => TYPER: ReadOnly<u64>),
        (0x0010 => _reserved_0),
        /// Power Management Control Register (GICR_WAKER)
        (0x0014 => WAKER: ReadWrite<u32>),
        (0x0018 => _reserved_1),
        /// Function Control Register (GICR_FCTLR)
        (0x0020 => FCTLR: ReadWrite<u32>),
        /// Power Register (GICR_PWRR)
        (0x0024 => PWRR: ReadWrite<u32>),
        /// Secure-only Class Register (GICR_CLASS)
        (0x0028 => CLASS: ReadWrite<u32>),
        (0x002C => _reserved_2),
        /// Set LPI Register (GICR_SETLPIR)
        (0x0040 => SETLPIR: WriteOnly<u64>),
        /// Clear LPI Register (GICR_CLRLPIR)
        (0x0048 => CLRLPIR: WriteOnly<u64>),
        (0x0050 => _reserved_3),
        /// Redistributor Properties Base Address Register (GICR_PROPBASER)
        (0x0070 => PROPBASER: ReadWrite<u64>),
        /// Redistributor LPI Pending Table Base Address Register (GICR_PENDBASER)
        (0x0078 => PENDBASER: ReadWrite<u64>),
        (0x0080 => _reserved_4),
        /// Invalidate LPI Register (GICR_INVLPIR)
        (0x00A0 => INVLPIR: WriteOnly<u64>),
        (0x00A8 => _reserved_5),
        /// Invalidate All LPI Register (GICR_INVALLR)
        (0x00B0 => INVALLR: WriteOnly<u64>),
        (0x00B8 => _reserved_6),
        /// Redistributor Synchronization Register (GICR_SYNCR)
        (0x00C0 => SYNCR: ReadOnly<u32>),
        (0x00C4 => _reserved_7),
        /// *eripheral ID Registers (GICR_PIDRn)
        (0xFFD0 => PIDR4: ReadOnly<u32>),
        (0xFFD4 => PIDR5: ReadOnly<u32>),
        (0xFFD8 => PIDR6: ReadOnly<u32>),
        (0xFFDC => PIDR7: ReadOnly<u32>),
        (0xFFE0 => PIDR0: ReadOnly<u32>),
        (0xFFE4 => PIDR1: ReadOnly<u32>),
        (0xFFE8 => PIDR2: ReadOnly<u32>),
        (0xFFEC => PIDR3: ReadOnly<u32>),
        /// Component ID Registers (GICR_CIDRn)
        (0xFFF0 => CIDR0: ReadOnly<u32>),
        (0xFFF4 => CIDR1: ReadOnly<u32>),
        (0xFFF8 => CIDR2: ReadOnly<u32>),
        (0xFFFC => CIDR3: ReadOnly<u32>),
        (0x10000 => @END),
    }
}



/// The GIC distributor.
///
/// The Distributor block performs interrupt prioritization and distribution
/// to the CPU interface blocks that connect to the processors in the system.
///
/// The Distributor provides a programming interface for:
/// - Globally enabling the forwarding of interrupts to the CPU interfaces.
/// - Enabling or disabling each interrupt.
/// - Setting the priority level of each interrupt.
/// - Setting the target processor list of each interrupt.
/// - Setting each peripheral interrupt to be level-sensitive or edge-triggered.
/// - Setting each interrupt as either Group 0 or Group 1.
/// - Forwarding an SGI to one or more target processors.
///
/// In addition, the Distributor provides:
/// - visibility of the state of each interrupt
/// - a mechanism for software to set or clear the pending state of a peripheral
///   interrupt.
pub struct GicDistributor {
    base: NonNull<GicDistributorRegs>,
    max_irqs: usize,
}

/// The GIC CPU interface.
///
/// Each CPU interface block performs priority masking and preemption
/// handling for a connected processor in the system.
///
/// Each CPU interface provides a programming interface for:
///
/// - enabling the signaling of interrupt requests to the processor
/// - acknowledging an interrupt
/// - indicating completion of the processing of an interrupt
/// - setting an interrupt priority mask for the processor
/// - defining the preemption policy for the processor
/// - determining the highest priority pending interrupt for the processor.
pub struct GicCpuInterface {
    base: NonNull<GicCpuInterfaceRegs>,
}

unsafe impl Send for GicDistributor {}
unsafe impl Sync for GicDistributor {}

unsafe impl Send for GicCpuInterface {}
unsafe impl Sync for GicCpuInterface {}

impl GicDistributor {
    /// Construct a new GIC distributor instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
            max_irqs: GIC_MAX_IRQ,
        }
    }

    const fn regs(&self) -> &GicDistributorRegs {
        unsafe { self.base.as_ref() }
    }

    /// The number of implemented CPU interfaces.
    pub fn cpu_num(&self) -> usize {
        ((self.regs().TYPER.get() as usize >> 5) & 0b111) + 1
    }

    /// The maximum number of interrupts that the GIC supports
    pub fn max_irqs(&self) -> usize {
        ((self.regs().TYPER.get() as usize & 0b11111) + 1) * 32
    }

    /// Configures the trigger mode for the given interrupt.
    pub fn configure_interrupt(&mut self, vector: usize, tm: TriggerMode) {
        // Only configurable for SPI interrupts
        if vector >= self.max_irqs || vector < SPI_RANGE.start {
            return;
        }

        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let reg_idx = vector >> 4;
        let bit_shift = ((vector & 0xf) << 1) + 1;
        let mut reg_val = self.regs().ICFGR[reg_idx].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }
        self.regs().ICFGR[reg_idx].set(reg_val);
    }

    /// Enables or disables the given interrupt.
    pub fn set_enable(&mut self, vector: usize, enable: bool) {
        if vector >= self.max_irqs {
            return;
        }
        let reg = vector / 32;
        let mask = 1 << (vector % 32);
        if enable {
            self.regs().ISENABLER[reg].set(mask);
        } else {
            self.regs().ICENABLER[reg].set(mask);
        }
    }

    /// Initializes the GIC distributor.
    ///
    /// It disables all interrupts, sets the target of all SPIs to CPU 0,
    /// configures all SPIs to be edge-triggered, and finally enables the GICD.
    ///
    /// This function should be called only once.
    pub fn init(&mut self) {
        let max_irqs = self.max_irqs();
        assert!(max_irqs <= GIC_MAX_IRQ);
        self.max_irqs = max_irqs;

        // Disable all interrupts
        for i in (0..max_irqs).step_by(32) {
            self.regs().ICENABLER[i / 32].set(u32::MAX);
            self.regs().ICPENDR[i / 32].set(u32::MAX);
        }
        if self.cpu_num() > 1 {
            for i in (SPI_RANGE.start..max_irqs).step_by(4) {
                // Set external interrupts to target cpu 0
                self.regs().ITARGETSR[i / 4].set(0x01_01_01_01);
            }
        }
        // Initialize all the SPIs to edge triggered
        for i in SPI_RANGE.start..max_irqs {
            self.configure_interrupt(i, TriggerMode::Edge);
        }

        // enable GIC0
        self.regs().CTLR.set(1);
    }
}

use log::info;

impl GicCpuInterface {
    /// Construct a new GIC CPU interface instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            base: NonNull::new(base).unwrap().cast(),
        }
    }

    /// Returns the interrupt ID of the highest priority pending interrupt for
    /// the CPU interface. (read GICC_IAR)
    ///
    /// The read returns a spurious interrupt ID of `1023` if the distributor
    /// or the CPU interface are disabled, or there is no pending interrupt on
    /// the CPU interface.
    pub fn iar(&self) -> u32 {
        read_sysreg!(icc_iar1_el1) as u32
    }

    /// Informs the CPU interface that it has completed the processing of the
    /// specified interrupt. (write GICC_EOIR)
    ///
    /// The value written must be the value returns from [`Self::iar`].
    pub fn eoi(&self, iar: u32) {
        write_sysreg!(icc_eoir1_el1, iar as u64);
    }

    /// handles the signaled interrupt.
    ///
    /// It first reads GICC_IAR to obtain the pending interrupt ID and then
    /// calls the given handler. After the handler returns, it writes GICC_EOIR
    /// to acknowledge the interrupt.
    ///
    /// If read GICC_IAR returns a spurious interrupt ID of `1023`, it does
    /// nothing.
    pub fn handle_irq<F>(&self, handler: F)
    where
        F: FnOnce(u32),
    {
        let iar = self.iar();
        let vector = iar & 0x3ff;
        if vector < 1020 {
            handler(vector);
            self.eoi(iar);
        } else {
            // spurious
        }
    }

    /// Initializes the GIC CPU interface.
    ///
    /// It unmask interrupts at all priority levels and enables the GICC.
    ///
    /// This function should be called only once.
    pub fn init(&self) {
        // enable GIC0
        let _ctlr = read_sysreg!(icc_ctlr_el1);
        write_sysreg!(icc_ctlr_el1, 0x1);
        // unmask interrupts at all priority levels
        let _pmr = read_sysreg!(icc_pmr_el1);
        write_sysreg!(icc_pmr_el1, 0xff);
        // Enable group 1 irq
        let _igrpen = read_sysreg!(icc_igrpen1_el1);
        write_sysreg!(icc_igrpen1_el1, 0x1);
    }
}
