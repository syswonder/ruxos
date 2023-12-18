/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef _SCHED_H
#define _SCHED_H

#include <stddef.h>
#include <sys/types.h>

#define CLONE_NEWTIME        0x00000080
#define CLONE_VM             0x00000100
#define CLONE_FS             0x00000200
#define CLONE_FILES          0x00000400
#define CLONE_SIGHAND        0x00000800
#define CLONE_PIDFD          0x00001000
#define CLONE_PTRACE         0x00002000
#define CLONE_VFORK          0x00004000
#define CLONE_PARENT         0x00008000
#define CLONE_THREAD         0x00010000
#define CLONE_NEWNS          0x00020000
#define CLONE_SYSVSEM        0x00040000
#define CLONE_SETTLS         0x00080000
#define CLONE_PARENT_SETTID  0x00100000
#define CLONE_CHILD_CLEARTID 0x00200000
#define CLONE_DETACHED       0x00400000
#define CLONE_UNTRACED       0x00800000
#define CLONE_CHILD_SETTID   0x01000000
#define CLONE_NEWCGROUP      0x02000000
#define CLONE_NEWUTS         0x04000000
#define CLONE_NEWIPC         0x08000000
#define CLONE_NEWUSER        0x10000000
#define CLONE_NEWPID         0x20000000
#define CLONE_NEWNET         0x40000000
#define CLONE_IO             0x80000000

typedef struct cpu_set_t {
    unsigned long __bits[128 / sizeof(long)];
} cpu_set_t;

#define __CPU_op_S(i, size, set, op)                                            \
    ((i) / 8U >= (size) ? 0                                                     \
                        : (((unsigned long *)(set))[(i) / 8 / sizeof(long)] op( \
                              1UL << ((i) % (8 * sizeof(long))))))

#define CPU_SET_S(i, size, set) __CPU_op_S(i, size, set, |=)
#define CPU_ZERO_S(size, set)   memset(set, 0, size)

#define CPU_SET(i, set) CPU_SET_S(i, sizeof(cpu_set_t), set);
#define CPU_ZERO(set)   CPU_ZERO_S(sizeof(cpu_set_t), set)

int sched_setaffinity(pid_t, size_t, const cpu_set_t *);

int sched_yield(void);

#endif // _SCHED_H
