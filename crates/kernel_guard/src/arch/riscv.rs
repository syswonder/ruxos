/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::arch::asm;

/// Bit 1: Supervisor Interrupt Enable
const SIE_BIT: usize = 1 << 1;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    // clear the `SIE` bit, and return the old CSR
    unsafe { asm!("csrrc {}, sstatus, {}", out(reg) flags, const SIE_BIT) };
    flags & SIE_BIT
}

#[inline]
pub fn local_irq_restore(flags: usize) {
    // restore the `SIE` bit
    unsafe { asm!("csrrs x0, sstatus, {}", in(reg) flags) };
}
