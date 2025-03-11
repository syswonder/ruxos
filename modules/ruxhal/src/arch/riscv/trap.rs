/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use riscv::register::scause::{self, Exception as E, Trap};

use super::{disable_irqs, enable_irqs, TrapFrame};

include_asm_marcos!();

core::arch::global_asm!(
    include_str!("trap.S"),
    trapframe_size = const core::mem::size_of::<TrapFrame>(),
);

fn handle_breakpoint(sepc: &mut usize) {
    debug!("Exception(Breakpoint) @ {:#x} ", sepc);
    *sepc += 2
}

#[no_mangle]
fn riscv_trap_handler(tf: &mut TrapFrame, _from_user: bool) {
    let scause = scause::read();
    match scause.cause() {
        Trap::Exception(E::Breakpoint) => handle_breakpoint(&mut tf.sepc),
        Trap::Interrupt(_) => crate::trap::handle_irq_extern(scause.bits()),
        #[cfg(feature = "musl")]
        Trap::Exception(E::UserEnvCall) => {
            #[cfg(feature = "irq")]
            enable_irqs();
            let ret = crate::trap::handle_syscall(
                tf.regs.a7,
                [
                    tf.regs.a0 as _,
                    tf.regs.a1 as _,
                    tf.regs.a2 as _,
                    tf.regs.a3 as _,
                    tf.regs.a4 as _,
                    tf.regs.a5 as _,
                ],
            );
            tf.regs.a0 = ret as _;
            #[cfg(feature = "irq")]
            disable_irqs();
        }
        #[cfg(feature = "paging")]
        Trap::Exception(E::LoadPageFault) => {
            let vaddr = riscv::register::stval::read();
            crate::trap::handle_page_fault(vaddr, crate::trap::PageFaultCause::READ);
        }
        #[cfg(feature = "paging")]
        Trap::Exception(E::StorePageFault) => {
            let vaddr = riscv::register::stval::read();
            crate::trap::handle_page_fault(vaddr, crate::trap::PageFaultCause::WRITE);
        }
        #[cfg(feature = "paging")]
        Trap::Exception(E::InstructionPageFault) => {
            let vaddr = riscv::register::stval::read();
            crate::trap::handle_page_fault(vaddr, crate::trap::PageFaultCause::INSTRUCTION);
        }
        _ => {
            panic!(
                "Unhandled trap {:?} @ {:#x}:\n{:#x?}",
                scause.cause(),
                tf.sepc,
                tf
            );
        }
    }
}
