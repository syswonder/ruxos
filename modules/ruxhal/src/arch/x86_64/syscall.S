
.section .text
.code64
.global x86_syscall_entry
x86_syscall_entry:

    push    r15
    push    r14
    push    r13
    push    r12
    push    r11
    push    r10
    push    r9
    push    r8
    push    rdi
    push    rsi
    push    rbp
    push    rbx
    push    rdx
    push    rcx

    mov     rcx, r10

    sub     rsp, 8
    mov     [rsp], r9

    mov     r9, r8
    mov     r8, rcx
    mov     rcx, rdx
    mov     rdx, rsi
    mov     rsi, rdi
    mov     rdi, rax

    call    x86_syscall_handler

    add     rsp, 8

    pop     rcx
    pop     rdx
    pop     rbx
    pop     rbp
    pop     rsi
    pop     rdi
    pop     r8
    pop     r9
    pop     r10
    pop     r11
    pop     r12
    pop     r13
    pop     r14
    pop     r15

    # restore rflags
    push    r11
    popfq

    jmp     rcx
