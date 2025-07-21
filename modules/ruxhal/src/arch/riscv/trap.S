.macro SAVE_REGS, from_user
    addi    sp, sp, -{trapframe_size}
    PUSH_GENERAL_REGS
    addi    t2, sp, {trapframe_size}
    csrr    t0, sepc
    csrr    t1, sstatus
    STR     t0, sp, 31                  // tf.sepc
    STR     t1, sp, 32                  // tf.sstatus
    STR     gp, sp, 2
    STR     t2, sp, 1                   // tf.regs.sp
    csrr   gp, sscratch
.if \from_user == 1
    LDR     t0, sp, 3                   // load supervisor tp
    STR     tp, sp, 3
    mv      tp, t0
.endif
.endm

.macro RESTORE_REGS, from_user
    // LDR     gp, sp, 2
.if \from_user == 1
    LDR     t0, sp, 3
    STR     tp, sp, 3                   // save supervisor tp
    mv      tp, t0
    addi    t0, sp, {trapframe_size}    // put supervisor sp to scratch
.endif
    LDR     gp, sp, 2
    LDR     t0, sp, 31
    LDR     t1, sp, 32
    csrw    sepc, t0
    csrw    sstatus, t1
    //csrw    sscratch, t2
    POP_GENERAL_REGS
    LDR     sp, sp, 1                   // load sp from tf.regs.sp
.endm

.section .text
.balign 4
.global trap_vector_base
trap_vector_base:
    // sscratch == 0: trap from S mode
    // sscratch != 0: trap from U mode
    //csrrw   sp, sscratch, sp            // switch sscratch and sp
    //bnez    sp, .Ltrap_entry_u
    //csrr    sp, sscratch                // put supervisor sp back
    j       .Ltrap_entry_s

.Ltrap_entry_s:
    SAVE_REGS 0
    mv      a0, sp
    li      a1, 0
    call    riscv_trap_handler
    RESTORE_REGS 0
    sret

.Ltrap_entry_u:
    SAVE_REGS 1
    mv      a0, sp
    li      a1, 1
    call    riscv_trap_handler
    RESTORE_REGS 1
    sret
