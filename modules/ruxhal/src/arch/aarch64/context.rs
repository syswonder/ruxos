/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::{
    arch::asm,
    fmt::{Debug, LowerHex},
};
use memory_addr::{PhysAddr, VirtAddr};

/// Saved registers when a trap (exception) occurs.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct TrapFrame {
    /// General-purpose registers (R0..R30).
    pub r: [u64; 31],
    /// User Stack Pointer (SP_EL0).
    pub usp: u64,
    /// Exception Link Register (ELR_EL1).
    pub elr: u64,
    /// Saved Process Status Register (SPSR_EL1).
    pub spsr: u64,
}

struct EnumerateReg<'a, T>(&'a [T]);

impl<'a, T: Debug + LowerHex> Debug for EnumerateReg<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut dbg_l = f.debug_list();
        for (i, r) in self.0.iter().enumerate() {
            dbg_l.entry(&format_args!("x{}: {:#x}", i, r));
        }
        dbg_l.finish()
    }
}

impl Debug for TrapFrame {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TrapFrame")
            .field("r", &EnumerateReg(&self.r))
            .field("usp", &self.usp)
            .field("elr", &self.elr)
            .field("spsr", &self.spsr)
            .finish()
    }
}

/// FP & SIMD registers.
#[repr(C, align(16))]
#[derive(Debug, Default)]
pub struct FpState {
    /// 128-bit SIMD & FP registers (V0..V31)
    pub regs: [u128; 32],
    /// Floating-point Control Register (FPCR)
    pub fpcr: u32,
    /// Floating-point Status Register (FPSR)
    pub fpsr: u32,
}

/// Saved hardware states of a task.
///
/// The context usually includes:
///
/// - Callee-saved registers
/// - Stack pointer register
/// - Thread pointer register (for thread-local storage, currently unsupported)
/// - FP/SIMD registers
///
/// On context switch, current task saves its context from CPU to memory,
/// and the next task restores its context from memory to CPU.
#[allow(missing_docs)]
#[repr(C)]
#[derive(Debug)]
pub struct TaskContext {
    pub sp: u64,
    pub tpidr_el0: u64,
    pub r19: u64,
    pub r20: u64,
    pub r21: u64,
    pub r22: u64,
    pub r23: u64,
    pub r24: u64,
    pub r25: u64,
    pub r26: u64,
    pub r27: u64,
    pub r28: u64,
    pub r29: u64,
    pub lr: u64, // r30
    #[cfg(feature = "fp_simd")]
    pub fp_state: FpState,
}

impl TaskContext {
    /// Creates a new default context for a new task.
    pub const fn new() -> Self {
        unsafe { core::mem::MaybeUninit::<Self>::zeroed().assume_init() }
    }

    /// Initializes the context for a new task, with the given entry point and
    /// kernel stack.
    pub fn init(&mut self, entry: usize, kstack_top: VirtAddr, tls_area: VirtAddr) {
        self.sp = kstack_top.as_usize() as u64;
        self.lr = entry as u64;
        self.tpidr_el0 = tls_area.as_usize() as u64;
    }

    /// Saves the current task's context from CPU to memory.
    ///
    /// # Safety
    ///
    /// - `src` must be a valid pointer to a memory region of at least `size` bytes.
    /// - `dst` must be a valid pointer to a memory region of at least `size` bytes.
    /// - The caller must ensure that no other thread or operation modifies the memory
    ///   at `src` or `dst` while this function is executing.
    /// - The size should not exceed the allocated memory size for `src` and `dst`.
    pub unsafe fn save_current_content(&mut self, src: *const u8, dst: *mut u8, size: usize) {
        unsafe {
            save_stack(src, dst, size);

            #[cfg(feature = "fp_simd")]
            save_fpstate_context(&mut self.fp_state);

            // will ret from here
            save_current_context(self);
        }
    }

    /// Switches to another task.
    ///
    /// It first saves the current task's context from CPU to this place, and then
    /// restores the next task's context from `next_ctx` to CPU.
    #[inline(never)]
    pub fn switch_to(&mut self, next_ctx: &Self, page_table_addr: PhysAddr) {
        unsafe {
            #[cfg(feature = "fp_simd")]
            fpstate_switch(&mut self.fp_state, &next_ctx.fp_state);
            // switch to the next process's page table, stack would be unavailable before context switch finished
            context_switch(self, next_ctx, page_table_addr.as_usize() as u64);
        }
    }
}

#[naked]
#[allow(named_asm_labels)]
// TODO: consider using SIMD instructions to copy the stack in parallel.
unsafe extern "C" fn save_stack(src: *const u8, dst: *mut u8, size: usize) {
    // x0: src, x1: dst, x2: size
    asm!(
        "
        mov x9, 0x0 // clear x9

        _copy_stack_start:
        cmp     x9, x2
        b.eq      _copy_stack_end
        ldr     x12, [x0]
        str     x12, [x1]
        add     x0, x0, 8
        add     x1, x1, 8
        add     x9, x9, 8
        b        _copy_stack_start
        _copy_stack_end:

        dsb  sy
        isb
        ret",
        options(noreturn),
    )
}

#[naked]
#[allow(named_asm_labels)]
unsafe extern "C" fn save_current_context(_current_task: &mut TaskContext) {
    asm!(
        "
        stp     x29, x30, [x0, 12 * 8]
        stp     x27, x28, [x0, 10 * 8]
        stp     x25, x26, [x0, 8 * 8]
        stp     x23, x24, [x0, 6 * 8]
        stp     x21, x22, [x0, 4 * 8]
        stp     x19, x20, [x0, 2 * 8]
        mrs     x20, tpidr_el0
        mov     x19, sp
        stp     x19, x20, [x0, 0 * 8]   // [x0] is parent's sp
        ldp     x19, x20, [x0, 2 * 8]
        isb
        ret",
        options(noreturn),
    )
}

#[naked]
#[cfg(feature = "fp_simd")]
unsafe extern "C" fn save_fpstate_context(_current_fpstate: &mut FpState) {
    asm!(
        "
        // save fp/neon context
        mrs     x9, fpcr
        mrs     x10, fpsr
        stp     q0, q1, [x0, 0 * 16]
        stp     q2, q3, [x0, 2 * 16]
        stp     q4, q5, [x0, 4 * 16]
        stp     q6, q7, [x0, 6 * 16]
        stp     q8, q9, [x0, 8 * 16]
        stp     q10, q11, [x0, 10 * 16]
        stp     q12, q13, [x0, 12 * 16]
        stp     q14, q15, [x0, 14 * 16]
        stp     q16, q17, [x0, 16 * 16]
        stp     q18, q19, [x0, 18 * 16]
        stp     q20, q21, [x0, 20 * 16]
        stp     q22, q23, [x0, 22 * 16]
        stp     q24, q25, [x0, 24 * 16]
        stp     q26, q27, [x0, 26 * 16]
        stp     q28, q29, [x0, 28 * 16]
        stp     q30, q31, [x0, 30 * 16]
        str     x9, [x0, 64 *  8]
        str     x10, [x0, 65 * 8]
        isb
        ret",
        options(noreturn),
    )
}

#[naked]
#[allow(named_asm_labels)]
unsafe extern "C" fn context_switch(
    _current_task: &mut TaskContext,
    _next_task: &TaskContext,
    _page_table_addr: u64,
) {
    asm!(
        "
        // save old context (callee-saved registers)
        stp     x29, x30, [x0, 12 * 8]
        stp     x27, x28, [x0, 10 * 8]
        stp     x25, x26, [x0, 8 * 8]
        stp     x23, x24, [x0, 6 * 8]
        stp     x21, x22, [x0, 4 * 8]
        stp     x19, x20, [x0, 2 * 8]
        mov     x19, sp
        mrs     x20, tpidr_el0
        stp     x19, x20, [x0]

        // switch to next task's page table
        mrs     x19, TTBR1_EL1
        cmp     x19, x2
        b.eq     _switch_page_table_done
        _switch_page_table:
        mov     x19, x2
        msr     TTBR1_EL1, x19
        tlbi vmalle1
        dsb sy
        isb
        // no need to switch page table, just continue
        _switch_page_table_done:

        // restore new context
        ldp     x19, x20, [x1]
        mov     sp, x19
        msr     tpidr_el0, x20
        ldp     x19, x20, [x1, 2 * 8]
        ldp     x21, x22, [x1, 4 * 8]
        ldp     x23, x24, [x1, 6 * 8]
        ldp     x25, x26, [x1, 8 * 8]
        ldp     x27, x28, [x1, 10 * 8]
        ldp     x29, x30, [x1, 12 * 8]

        isb
        ret",
        options(noreturn),
    )
}

#[naked]
#[cfg(feature = "fp_simd")]
unsafe extern "C" fn fpstate_switch(_current_fpstate: &mut FpState, _next_fpstate: &FpState) {
    asm!(
        "
        // save fp/neon context
        mrs     x9, fpcr
        mrs     x10, fpsr
        stp     q0, q1, [x0, 0 * 16]
        stp     q2, q3, [x0, 2 * 16]
        stp     q4, q5, [x0, 4 * 16]
        stp     q6, q7, [x0, 6 * 16]
        stp     q8, q9, [x0, 8 * 16]
        stp     q10, q11, [x0, 10 * 16]
        stp     q12, q13, [x0, 12 * 16]
        stp     q14, q15, [x0, 14 * 16]
        stp     q16, q17, [x0, 16 * 16]
        stp     q18, q19, [x0, 18 * 16]
        stp     q20, q21, [x0, 20 * 16]
        stp     q22, q23, [x0, 22 * 16]
        stp     q24, q25, [x0, 24 * 16]
        stp     q26, q27, [x0, 26 * 16]
        stp     q28, q29, [x0, 28 * 16]
        stp     q30, q31, [x0, 30 * 16]
        str     x9, [x0, 64 *  8]
        str     x10, [x0, 65 * 8]

        // restore fp/neon context
        ldp     q0, q1, [x1, 0 * 16]
        ldp     q2, q3, [x1, 2 * 16]
        ldp     q4, q5, [x1, 4 * 16]
        ldp     q6, q7, [x1, 6 * 16]
        ldp     q8, q9, [x1, 8 * 16]
        ldp     q10, q11, [x1, 10 * 16]
        ldp     q12, q13, [x1, 12 * 16]
        ldp     q14, q15, [x1, 14 * 16]
        ldp     q16, q17, [x1, 16 * 16]
        ldp     q18, q19, [x1, 18 * 16]
        ldp     q20, q21, [x1, 20 * 16]
        ldp     q22, q23, [x1, 22 * 16]
        ldp     q24, q25, [x1, 24 * 16]
        ldp     q26, q27, [x1, 26 * 16]
        ldp     q28, q29, [x1, 28 * 16]
        ldp     q30, q31, [x1, 30 * 16]
        ldr     x9, [x1, 64 * 8]
        ldr     x10, [x1, 65 * 8]
        msr     fpcr, x9
        msr     fpsr, x10

        isb
        ret",
        options(noreturn),
    )
}
