/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::arch::global_asm;

#[cfg(all(feature = "irq", feature = "musl"))]
use crate::arch::{disable_irqs, enable_irqs};
#[cfg(feature = "paging")]
use crate::trap::PageFaultCause;
use aarch64_cpu::registers::{ESR_EL1, FAR_EL1};
use tock_registers::interfaces::Readable;

use super::TrapFrame;

global_asm!(include_str!("trap.S"));

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapKind {
    Synchronous = 0,
    Irq = 1,
    Fiq = 2,
    SError = 3,
}

#[repr(u8)]
#[derive(Debug)]
#[allow(dead_code)]
enum TrapSource {
    CurrentSpEl0 = 0,
    CurrentSpElx = 1,
    LowerAArch64 = 2,
    LowerAArch32 = 3,
}

#[no_mangle]
fn invalid_exception(tf: &TrapFrame, kind: TrapKind, source: TrapSource) {
    panic!(
        "Invalid exception {:?} from {:?}:\n{:#x?}",
        kind, source, tf
    );
}

#[no_mangle]
fn handle_sync_exception(tf: &mut TrapFrame) {
    let esr = ESR_EL1.extract();
    match esr.read_as_enum(ESR_EL1::EC) {
        Some(ESR_EL1::EC::Value::Brk64) => {
            let iss = esr.read(ESR_EL1::ISS);
            debug!("BRK #{:#x} @ {:#x} ", iss, tf.elr);
            tf.elr += 4;
        }
        #[cfg(feature = "musl")]
        Some(ESR_EL1::EC::Value::SVC64) => {
            debug!("Handle supervisor call {}", tf.r[8]);
            #[cfg(feature = "irq")]
            enable_irqs();
            let result = crate::trap::handle_syscall(
                tf.r[8] as usize,
                [
                    tf.r[0] as _,
                    tf.r[1] as _,
                    tf.r[2] as _,
                    tf.r[3] as _,
                    tf.r[4] as _,
                    tf.r[5] as _,
                ],
            );
            tf.r[0] = result as u64;
            #[cfg(feature = "irq")]
            disable_irqs();
        }
        Some(ESR_EL1::EC::Value::DataAbortLowerEL)
        | Some(ESR_EL1::EC::Value::InstrAbortLowerEL) => {
            let iss = esr.read(ESR_EL1::ISS);
            warn!(
                "EL0 Page Fault @ {:#x}, FAR={:#x}, ISS={:#x}",
                tf.elr,
                FAR_EL1.get(),
                iss
            );
        }
        Some(ESR_EL1::EC::Value::DataAbortCurrentEL)
        | Some(ESR_EL1::EC::Value::InstrAbortCurrentEL) => {
            let iss = esr.read(ESR_EL1::ISS);
            #[cfg(feature = "paging")]
            {
                let vaddr = FAR_EL1.get() as usize;

                // this cause is coded like linux.
                let cause: PageFaultCause = match esr.read_as_enum(ESR_EL1::EC) {
                    Some(ESR_EL1::EC::Value::DataAbortCurrentEL) if iss & 0x40 != 0 => {
                        PageFaultCause::WRITE // = store
                    }
                    Some(ESR_EL1::EC::Value::DataAbortCurrentEL) if iss & 0x40 == 0 => {
                        PageFaultCause::READ //  = load
                    }
                    _ => {
                        PageFaultCause::INSTRUCTION // = instruction fetch
                    }
                };
                let is_mapped = crate::trap::handle_page_fault(vaddr, cause);

                if is_mapped {
                    return;
                }
                error!(
                    "Page fault @ {:#x}, cause={:?}, is_mapped={}",
                    tf.elr, cause, is_mapped
                );
            }
            panic!(
                "EL1 Page Fault @ {:#x}, FAR={:#x}, ISS={:#x}:\n{:#x?}",
                tf.elr,
                FAR_EL1.get(),
                iss,
                tf,
            );
        }
        _ => {
            panic!(
                "Unhandled synchronous exception @ {:#x}: ESR={:#x} (EC {:#08b}, ISS {:#x})",
                tf.elr,
                esr.get(),
                esr.read(ESR_EL1::EC),
                esr.read(ESR_EL1::ISS),
            );
        }
    }
    #[cfg(feature = "signal")]
    {
        crate::trap::handle_signal();
    }
}

#[no_mangle]
fn handle_irq_exception(_tf: &TrapFrame) {
    crate::trap::handle_irq_extern(0)
}
