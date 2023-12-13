/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef _SETJMP_H
#define _SETJMP_H

#include <features.h>

#if defined(__aarch64__)
typedef unsigned long __jmp_buf[22];
#elif defined(__riscv__) || defined(__riscv)
typedef unsigned long __jmp_buf[26];
#elif defined(__x86_64__)
typedef unsigned long __jmp_buf[8];
#endif

typedef struct __jmp_buf_tag {
    __jmp_buf __jb;
    unsigned long __fl;
    unsigned long __ss[128 / sizeof(long)];
} jmp_buf[1];

int setjmp(jmp_buf);
_Noreturn void longjmp(jmp_buf, int);

#endif
