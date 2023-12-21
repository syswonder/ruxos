/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::arch::asm;

/// Interrupt Enable Flag (IF)
const IF_BIT: usize = 1 << 9;

#[inline]
pub fn local_irq_save_and_disable() -> usize {
    let flags: usize;
    unsafe { asm!("pushf; pop {}; cli", out(reg) flags) };
    flags & IF_BIT
}

#[inline]
pub fn local_irq_restore(flags: usize) {
    if flags != 0 {
        unsafe { asm!("sti") };
    } else {
        unsafe { asm!("cli") };
    }
}
