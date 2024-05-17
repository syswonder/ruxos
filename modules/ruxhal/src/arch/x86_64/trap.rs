/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use x86::{controlregs::cr2, irq::*};

use super::context::TrapFrame;
#[cfg(all(feature = "paging", feature = "irq", feature = "smp"))]
use crate::arch::{flush_tlb_ipi_handler, INVALID_TLB_VECTOR};
#[cfg(any(
    all(feature = "paging", feature = "irq", feature = "smp"),
    all(feature = "paging", not(feature = "smp"))
))]
use crate::trap::PageFaultCause;

core::arch::global_asm!(include_str!("trap.S"));

const IRQ_VECTOR_START: u8 = 0x20;
const IRQ_VECTOR_END: u8 = 0xff;

#[no_mangle]
fn x86_trap_handler(tf: &TrapFrame) {
    match tf.vector as u8 {
        PAGE_FAULT_VECTOR => {
            if tf.is_user() {
                warn!(
                    "User #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}",
                    tf.rip,
                    unsafe { cr2() },
                    tf.error_code,
                );
            } else {
                let vaddr = unsafe { cr2() };
                #[cfg(any(
                    all(feature = "paging", feature = "irq", feature = "smp"),
                    all(feature = "paging", not(feature = "smp"))
                ))]
                {
                    // this cause is coded like linux.
                    let cause: PageFaultCause = match tf.error_code {
                        x if x & 0x10 != 0 => PageFaultCause::INSTRUCTION,
                        x if x & 0x02 != 0 => PageFaultCause::WRITE,
                        _ => PageFaultCause::READ,
                    };
                    if crate::trap::handle_page_fault(vaddr, cause) {
                        return;
                    }
                }
                panic!(
                    "Kernel #PF @ {:#x}, fault_vaddr={:#x}, error_code={:#x}:\n{:#x?}",
                    tf.rip, vaddr, tf.error_code, tf,
                );
            }
        }
        BREAKPOINT_VECTOR => debug!("#BP @ {:#x} ", tf.rip),
        GENERAL_PROTECTION_FAULT_VECTOR => {
            panic!(
                "#GP @ {:#x}, error_code={:#x}:\n{:#x?}",
                tf.rip, tf.error_code, tf
            );
        }
        #[cfg(all(feature = "paging", feature = "irq", feature = "smp"))]
        INVALID_TLB_VECTOR => flush_tlb_ipi_handler(),
        IRQ_VECTOR_START..=IRQ_VECTOR_END => crate::trap::handle_irq_extern(tf.vector as _),
        _ => {
            panic!(
                "Unhandled exception {} (error_code = {:#x}) @ {:#x}:\n{:#x?}",
                tf.vector, tf.error_code, tf.rip, tf
            );
        }
    }
}

#[cfg(feature = "musl")]
#[no_mangle]
fn x86_syscall_handler(
    syscall_id: usize,
    arg1: usize,
    arg2: usize,
    arg3: usize,
    arg4: usize,
    arg5: usize,
    arg6: usize,
) -> isize {
    debug!(
        "syscall_id: {}, 
        arg1: {:#x}, arg2: {:#x}, arg3:{:#x}, arg4: {:#x}, arg5:{:#x}, arg6: {:#x}",
        syscall_id, arg1, arg2, arg3, arg4, arg5, arg6
    );
    crate::trap::handle_syscall(syscall_id, [arg1, arg2, arg3, arg4, arg5, arg6])
}
