.include "syscall_asm.inc"

.global __unmapself
.type __unmapself, %function
__unmapself:
	li a7, 215 # SYS_munmap
	RUX_SYSCALL_ASM
	li a7, 93  # SYS_exit
	RUX_SYSCALL_ASM
