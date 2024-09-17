/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

mod context;
mod gdt;
mod idt;

mod trap;

use core::arch::asm;

use memory_addr::{PhysAddr, VirtAddr};
use x86::{controlregs, msr, tlb};
use x86_64::instructions::interrupts;

#[cfg(feature = "musl")]
use x86_64::registers::model_specific::EferFlags;

#[cfg(feature = "irq")]
extern crate alloc;

#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
use {
    crate::{
        cpu::this_cpu_id,
        platform::irq::{end_of_interrupt, send_ipi_excluding_self},
    },
    alloc::vec::Vec,
    ruxconfig::SMP,
    spinlock::SpinRaw,
};

pub use self::context::{ExtendedState, FxsaveArea, TaskContext, TrapFrame};
pub use self::gdt::GdtStruct;
pub use self::idt::IdtStruct;
pub use x86_64::structures::tss::TaskStateSegment;

/// Allows the current CPU to respond to interrupts.
#[inline]
pub fn enable_irqs() {
    #[cfg(target_os = "none")]
    interrupts::enable()
}

/// Makes the current CPU to ignore interrupts.
#[inline]
pub fn disable_irqs() {
    #[cfg(target_os = "none")]
    interrupts::disable()
}

/// Returns whether the current CPU is allowed to respond to interrupts.
#[inline]
pub fn irqs_enabled() -> bool {
    interrupts::are_enabled()
}

/// Relaxes the current CPU and waits for interrupts.
///
/// It must be called with interrupts enabled, otherwise it will never return.
#[inline]
pub fn wait_for_irqs() {
    if cfg!(target_os = "none") {
        unsafe { asm!("hlt") }
    } else {
        core::hint::spin_loop()
    }
}

/// Halt the current CPU.
#[inline]
pub fn halt() {
    disable_irqs();
    wait_for_irqs(); // should never return
}

/// Reads the register that stores the current page table root.
///
/// Returns the physical address of the page table root.
#[inline]
pub fn read_page_table_root() -> PhysAddr {
    PhysAddr::from(unsafe { controlregs::cr3() } as usize).align_down_4k()
}

/// Writes the register to update the current page table root.
///
/// # Safety
///
/// This function is unsafe as it changes the virtual memory address space.
pub unsafe fn write_page_table_root(root_paddr: PhysAddr) {
    let old_root = read_page_table_root();
    trace!("set page table root: {:#x} => {:#x}", old_root, root_paddr);
    if old_root != root_paddr {
        controlregs::cr3_write(root_paddr.as_usize() as _)
    }
}

/// data structure for communicating between IPI for the TLB flushing.
///
/// `Vaddr` means flushing the TLB entry that maps the given virtual address.
/// `All` means flushing the entire TLB.
#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
enum FlushTlbIpiData {
    Vaddr(usize),
    All,
}

// const variable for every CPU's flushing addresses.
#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
#[allow(clippy::declare_interior_mutable_const)]
const FLUSH_QUEUES: SpinRaw<Vec<FlushTlbIpiData>> = SpinRaw::new(Vec::new());
/// static variable for communicating between IPI for the TLB flushing.
#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
static FLUSHING_ADDRESSES: [SpinRaw<Vec<FlushTlbIpiData>>; SMP] = [FLUSH_QUEUES; SMP];
/// const irq vector for TLB flushing IPI.
#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
pub const INVALID_TLB_VECTOR: u8 = 0xff; // SPURIOUS APIC INTERRUPT

/// Flushes the TLB.
///
/// If `vaddr` is [`None`], flushes the entire TLB. Otherwise, flushes the TLB
/// entry that maps the given virtual address.
#[inline]
pub fn flush_tlb(vaddr: Option<VirtAddr>) {
    if let Some(vaddr) = vaddr {
        trace!("flush TLB entry: {:#x}", vaddr);
        unsafe {
            tlb::flush(vaddr.into());
        }
        #[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
        {
            for (i, flushing_vec) in FLUSHING_ADDRESSES.iter().enumerate().take(SMP) {
                if i != this_cpu_id() {
                    flushing_vec
                        .lock()
                        .push(FlushTlbIpiData::Vaddr(vaddr.into()));
                }
            }
            unsafe {
                send_ipi_excluding_self(INVALID_TLB_VECTOR);
            }
        }
    } else {
        trace!("flush all TLB entry");
        unsafe {
            tlb::flush_all();
        }
        #[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
        {
            for (i, flushing_vec) in FLUSHING_ADDRESSES.iter().enumerate().take(SMP) {
                if i != this_cpu_id() {
                    let mut flushing_addresses = flushing_vec.lock();
                    // clear all flushing addresses to avoid flushing again in IPI handler.
                    flushing_addresses.clear();
                    flushing_addresses.push(FlushTlbIpiData::All);
                }
            }
            unsafe {
                send_ipi_excluding_self(INVALID_TLB_VECTOR);
            }
        }
    }
}

/// Flushes the TLB in IPI handler.
///
/// This function is called in IPI handler, and it flushes the TLB entry that maps the given virtual address.
#[inline]
#[cfg(all(feature = "irq", feature = "paging", feature = "smp"))]
pub(crate) fn flush_tlb_ipi_handler() {
    // error!("flush TLB entry in IPI handler");
    let guard = kernel_guard::NoPreempt::new();
    unsafe {
        let mut flushing_addresses = FLUSHING_ADDRESSES[this_cpu_id()].lock();
        while let Some(flush_data) = flushing_addresses.pop() {
            if let FlushTlbIpiData::Vaddr(vaddr) = flush_data {
                tlb::flush(vaddr);
            } else {
                tlb::flush_all();
                flushing_addresses.clear();
            }
        }
        end_of_interrupt();
    }
    drop(guard);
}

/// Reads the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
#[inline]
pub fn read_thread_pointer() -> usize {
    unsafe { msr::rdmsr(msr::IA32_FS_BASE) as usize }
}

/// Writes the thread pointer of the current CPU.
///
/// It is used to implement TLS (Thread Local Storage).
///
/// # Safety
///
/// This function is unsafe as it changes the CPU states.
#[inline]
pub unsafe fn write_thread_pointer(fs_base: usize) {
    unsafe { msr::wrmsr(msr::IA32_FS_BASE, fs_base as u64) }
}

#[cfg(feature = "musl")]
core::arch::global_asm!(include_str!("syscall.S"),);

/// Set syscall entry
///
/// # Safety
///
/// This function modify MSR registers
#[cfg(feature = "musl")]
#[inline]
pub unsafe fn init_syscall_entry() {
    extern "C" {
        fn x86_syscall_entry();
    }

    let cpuid = raw_cpuid::CpuId::new();

    unsafe {
        assert!(cpuid
            .get_extended_processor_and_feature_identifiers()
            .unwrap()
            .has_syscall_sysret());

        x86_64::registers::model_specific::LStar::write(x86_64::VirtAddr::new(
            x86_syscall_entry as usize,
        ));
        x86_64::registers::model_specific::Efer::update(|efer| {
            efer.insert(
                EferFlags::SYSTEM_CALL_EXTENSIONS
                    | EferFlags::LONG_MODE_ACTIVE
                    | EferFlags::LONG_MODE_ENABLE,
            );
        });
        msr::wrmsr(msr::IA32_STAR, (0x10 << 48) | (0x10 << 32));
        msr::wrmsr(msr::IA32_FMASK, 0x47700);
    }
}
