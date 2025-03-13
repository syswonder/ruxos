/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Types and definitions for GICv3.
//!
//! The official documentation: <https://developer.arm.com/documentation/ihi0048/latest/>

use core::ptr::NonNull;

use crate::{TriggerMode, GIC_MAX_IRQ, SPI_RANGE};
use tock_registers::interfaces::{Readable, Writeable};
use tock_registers::register_structs;
use tock_registers::registers::{ReadOnly, ReadWrite, WriteOnly};

/// Reads and returns the value of the given aarch64 system register.
/// use crate::arch::sysreg::write_sysreg;
/// unsafe {write_sysreg!(icc_sgi1r_el1, val);}
/// let intid = unsafe { read_sysreg!(icc_iar1_el1) } as u32;
macro_rules! read_sysreg {
    ($name:ident) => {
        {
            let mut value: u64;
            unsafe{::core::arch::asm!(
                concat!("mrs {value:x}, ", ::core::stringify!($name)),
                value = out(reg) value,
                options(nomem, nostack),
            );}
            value
        }
    }
}
pub(crate) use read_sysreg;

/// Writes the given value to the given aarch64 system register.
macro_rules! write_sysreg {
    ($name:ident, $value:expr) => {
        {
            let v: u64 = $value;
            unsafe{::core::arch::asm!(
                concat!("msr ", ::core::stringify!($name), ", {value:x}"),
                value = in(reg) v,
                options(nomem, nostack),
            )}
        }
    }
}
pub(crate) use write_sysreg;

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
        (0x6100 => IROUTER: [ReadWrite<u64>; 988]),
        (0x7FE0 => _reserved_3),
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
    /// GIC Redistributor Registers
    #[allow(non_snake_case)]
    GICv3RdistLpisIf {
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

register_structs! {
    /// GIC Redistributor Registers
    #[allow(non_snake_case)]
    GICv3RdistSgisIf {
        (0x0000 => _reserved_0),
        (0x0080 => IGROUPR0: ReadWrite<u32>),
        (0x0084 => _reserved_4),
        /// Interrupt Set-Enable Register
        (0x0100 => ISENABLER0: ReadWrite<u32>),
        (0x0104 => _reserved_8),
        /// Interrupt Clear-Enable Register
        (0x0180 => ICENABLER0: ReadWrite<u32>),
        (0x0184 => _reserved_9),
        /// Interrupt Set-Pending Register
        (0x0200 => ISPENDR0: ReadWrite<u32>),
        (0x0204 => _reserved_10),
        /// Interrupt Clear-Pending Register
        (0x0280 => ICPENDR0: ReadWrite<u32>),
        (0x0284 => _reserved_11),
        /// Interrupt Set-Active Register
        (0x0300 => ISACTIVER0: ReadWrite<u32>),
        (0x0304 => _reserved_12),
        /// Interrupt Clear-Active Register
        (0x0380 => ICACTIVER0: ReadWrite<u32>),
        (0x0384 => _reserved_13),
        /// Interrupt Priority Registers (multiple entries)
        (0x0400 => IPRIORITYR: [ReadWrite<u32>; 8]),
        (0x0420 => _reserved_14),
        /// Interrupt Configuration Register
        (0x0C00 => ICFGR: [ReadWrite<u32>;2]),
        (0x0C08 => _reserved_15),
        /// Interrupt Group Modifier Register
        (0x0D00 => IGRPMODR0: ReadWrite<u32>),
        (0x0D04 => _reserved_16),
        /// Non-secure Access Control Register
        (0x0E00 => NSACR: ReadWrite<u32>),
        (0x0E04 => _reserved_17),
        /// Miscellaneous Status Register
        (0xC000 => MISCSTATUSR: ReadOnly<u32>),
        (0xC004 => _reserved_18),
        /// Interrupt Error Valid Register
        (0xC008 => IERRVR: ReadOnly<u32>),
        (0xC00C => _reserved_19),
        /// SGI Default Register
        (0xC010 => SGIDR: ReadWrite<u64>),
        (0xC018 => _reserved_20),
        /// Configuration ID0 Register
        (0xF000 => CFGID0: ReadOnly<u32>),
        /// Configuration ID1 Register
        (0xF004 => CFGID1: ReadOnly<u32>),
        (0xF008 => _reserved_21),
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

/// The GIC redistributor.
///
/// The Redistributor block is responsible for managing per-core interrupts,
/// including Software Generated Interrupts (SGIs), Private Peripheral Interrupts (PPIs),
/// and Locality-specific Peripheral Interrupts (LPIs). Each CPU core has its own
/// instance of the Redistributor.
///
/// The Redistributor provides a programming interface for:
/// - Managing the power state of the Redistributor for each CPU core.
/// - Enabling or disabling SGIs and PPIs.
/// - Setting the priority level of SGIs and PPIs.
/// - Configuring the trigger mode (level-sensitive or edge-triggered) for SGIs and PPIs.
/// - Managing Locality-specific Peripheral Interrupts (LPIs), which are designed for
///   scalable interrupt handling.
/// - Configuring the Redistributor's memory-mapped structures for LPI storage.
///
/// In addition, the Redistributor provides:
/// - A mechanism to wake up a CPU from power-saving states when an interrupt occurs.
/// - Controls to enable or disable specific interrupts at the core level.
/// - Support for Interrupt Translation Services (ITS) for LPI handling in large systems.

pub struct GicRedistributor {
    lpis: NonNull<GICv3RdistLpisIf>,
    sgis: NonNull<GICv3RdistSgisIf>,
}

unsafe impl Send for GicDistributor {}
unsafe impl Sync for GicDistributor {}

unsafe impl Send for GicRedistributor {}
unsafe impl Sync for GicRedistributor {}

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
        let val: u32;
        let mut irq_number: usize;

        // Disable the distributor
        //self.disable_dist();

        // Get GIC redistributor interface
        val = self.regs().TYPER.get();
        irq_number = ((((val) & 0x1f) + 1) << 5) as usize;
        if irq_number > GIC_MAX_IRQ {
            irq_number = GIC_MAX_IRQ + 1;
        }

        // Configure all SPIs as non-secure Group 1
        for i in (SPI_RANGE.start..irq_number).step_by(32 as usize) {
            self.regs().IGROUPR[i / 32].set(0xffffffff);
        }

        // Route all global SPIs to this CPU
        let mpidr: u64 = read_sysreg!(MPIDR_EL1);
        let aff = ((mpidr & 0xff00000000) >> 8) | //Affinity AFF3 bit mask
                (mpidr & 0x0000ff0000) | // Affinity AFF2 bit mask
                (mpidr & 0x000000ff00) | // Affinity AFF1 bit mask
                (mpidr & 0x00000000ff); // Affinity AFF0 bit mask
        let irouter_val = ((aff << 8) & 0xff00000000) | (aff & 0xffffff) | (0 << 31);

        for i in SPI_RANGE.start..irq_number {
            self.regs().IROUTER[i].set(irouter_val);
        }

        // Set all SPI's interrupt type to be level-sensitive
        for i in (SPI_RANGE.start..irq_number).step_by(16 as usize) {
            self.regs().ICFGR[i / 16].set(0);
        }

        // Set all SPI's priority to a default value
        for i in (SPI_RANGE.start..irq_number).step_by(4 as usize) {
            self.regs().IPRIORITYR[i / 4].set(0x80808080);
        }

        // Deactivate and disable all SPIs
        for i in (SPI_RANGE.start..irq_number).step_by(32 as usize) {
            self.regs().ICACTIVER[i / 32].set(0xffffffff);
            self.regs().ICENABLER[i / 32].set(0xffffffff);
        }

        // Wait for completion
        while self.regs().CTLR.get() & (1 << 31) != 0 {}

        // Enable the distributor
        self.regs().CTLR.set((1 << 4) | (1 << 1) | (1 << 0));
    }
}

impl GicRedistributor {
    /// Construct a new GIC redistributor instance from the base address.
    pub const fn new(base: *mut u8) -> Self {
        Self {
            lpis: NonNull::new(base).unwrap().cast(),
            sgis: NonNull::new(base.wrapping_add(0x010000)).unwrap().cast(),
        }
    }

    const fn lpis(&self) -> &GICv3RdistLpisIf {
        unsafe { self.lpis.as_ref() }
    }

    const fn sgis(&self) -> &GICv3RdistSgisIf {
        unsafe { self.sgis.as_ref() }
    }

    /// Reads the Interrupt Acknowledge Register.
    ///
    /// This retrieves the IRQ number of the highest-priority pending interrupt.
    pub fn iar(&self) -> u32 {
        read_sysreg!(icc_iar1_el1) as u32
    }

    /// Writes to the End of Interrupt Register.
    ///
    /// Marks the interrupt as handled.
    pub fn eoi(&self, iar: u32) {
        write_sysreg!(icc_eoir1_el1, iar as u64);
    }

    /// Writes to the Deactivate Interrupt Register.
    ///
    /// Ensures that the interrupt is fully deactivated in the GIC.
    pub fn dir(&self, iar: u32) {
        write_sysreg!(icc_dir_el1, iar as u64);
    }

    /// Handles an interrupt by invoking the provided handler function.
    ///
    /// # Arguments
    /// - `handler`: A function that takes an IRQ number and processes it.
    pub fn handle_irq<F>(&mut self, handler: F)
    where
        F: FnOnce(u32),
    {
        let iar = self.iar();
        let vector = iar & 0x3ff;
        if vector < 1020 {
            handler(vector);
            self.eoi(iar);
            self.dir(iar);
        } else {
            // spurious
        }
    }

    /// Configures the trigger mode for the given interrupt.
    pub fn configure_interrupt(&mut self, vector: usize, tm: TriggerMode) {
        // Only configurable for SPI interrupts
        if vector >= 32 {
            return;
        }

        // type is encoded with two bits, MSB of the two determine type
        // 16 irqs encoded per ICFGR register
        let reg_idx = vector >> 4;
        let bit_shift = ((vector & 0xf) << 1) + 1;
        let mut reg_val = self.sgis().ICFGR[reg_idx].get();
        match tm {
            TriggerMode::Edge => reg_val |= 1 << bit_shift,
            TriggerMode::Level => reg_val &= !(1 << bit_shift),
        }
        self.sgis().ICFGR[reg_idx].set(reg_val);
    }

    /// Enables or disables the given SGI or PPI interrupt.
    pub fn set_enable(&mut self, vector: usize, enable: bool) {
        if vector >= 32 {
            return;
        }
        let mask = 1 << (vector % 32);
        if enable {
            self.sgis().ISENABLER0.set(mask);
        } else {
            self.sgis().ICENABLER0.set(mask);
        }
    }

    /// Initializes the GIC Redistributor.
    ///
    /// This function:
    /// - Wakes up the Redistributor from power-saving mode.
    /// - Sets default interrupt priorities.
    /// - Disables all SGIs and PPIs.
    /// - Configures the GICv3 CPU interface for handling interrupts.
    ///
    /// This function should be executed for each CPU before enabling LPIs.
    pub fn init(&mut self) {
        let waker = self.lpis().WAKER.get();
        self.lpis().WAKER.set(waker & !0x02);
        while self.lpis().WAKER.get() & 0x04 != 0 {}
        for i in 0..8 {
            self.sgis().IPRIORITYR[i].set(0x80808080);
        }

        // Disable all SGIs and PPIs
        self.sgis().ICACTIVER0.set(0xffffffff);
        self.sgis().ICENABLER0.set(0xffff0000);
        self.sgis().IGROUPR0.set(0xffffffff);
        self.sgis().ISENABLER0.set(0xffff);

        while self.lpis().CTLR.get() & (1 << 31) != 0 {}

        let sre = read_sysreg!(icc_sre_el1);
        write_sysreg!(icc_sre_el1, sre | 0x7);

        write_sysreg!(icc_bpr1_el1, 0);

        let _pmr = read_sysreg!(icc_pmr_el1);
        write_sysreg!(icc_pmr_el1, 0xff);
        // enable GIC0
        let _ctlr = read_sysreg!(icc_ctlr_el1);
        write_sysreg!(icc_ctlr_el1, 0x2);
        // unmask interrupts at all priority levels
        // Enable group 1 irq
        let _igrpen = read_sysreg!(icc_igrpen1_el1);
        write_sysreg!(icc_igrpen1_el1, 0x1);

        let _cntp = read_sysreg!(CNTP_CTL_EL0);
        write_sysreg!(CNTP_CTL_EL0, 1);
    }
}
