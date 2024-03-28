.macro RUX_SYSCALL_ASM
    addi sp,sp,-8
    sd ra,0(sp) # might be able to store ra in sscratch?
    call riscv_syscall_asm
    ld ra,0(sp)
    addi sp,sp,8
.endm
