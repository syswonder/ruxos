#define __SYSCALL_LL_E(x) (x)
#define __SYSCALL_LL_O(x) (x)

extern long riscv_syscall_asm(long a0, long a1, long a2, long a3, long a4, long a5, long _,
                              long a7);

static inline long __syscall0(long n)
{
    return riscv_syscall_asm(0, 0, 0, 0, 0, 0, 0, n);
}

static inline long __syscall1(long n, long a)
{
    return riscv_syscall_asm(a, 0, 0, 0, 0, 0, 0, n);
}

static inline long __syscall2(long n, long a, long b)
{
    return riscv_syscall_asm(a, b, 0, 0, 0, 0, 0, n);
}

static inline long __syscall3(long n, long a, long b, long c)
{
    return riscv_syscall_asm(a, b, c, 0, 0, 0, 0, n);
}

static inline long __syscall4(long n, long a, long b, long c, long d)
{
    return riscv_syscall_asm(a, b, c, d, 0, 0, 0, n);
}

static inline long __syscall5(long n, long a, long b, long c, long d, long e)
{
    return riscv_syscall_asm(a, b, c, d, e, 0, 0, n);
}

static inline long __syscall6(long n, long a, long b, long c, long d, long e, long f)
{
    return riscv_syscall_asm(a, b, c, d, e, f, 0, n);
}

#define VDSO_USEFUL
/* We don't have a clock_gettime function.
#define VDSO_CGT_SYM "__vdso_clock_gettime"
#define VDSO_CGT_VER "LINUX_2.6" */

#define IPC_64 0
