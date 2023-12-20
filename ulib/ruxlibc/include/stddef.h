/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef __STDDEF_H__
#define __STDDEF_H__

#include <stdint.h>
typedef long clock_t;
typedef int clockid_t;

typedef uintptr_t size_t;
#include <sys/types.h>

/* size_t is used for memory object sizes */
typedef intptr_t ssize_t;
typedef ssize_t ptrdiff_t;



#define _Int64 long

typedef unsigned _Int64 fsblkcnt_t;
typedef unsigned _Int64 fsfilcnt_t;

#ifdef __cplusplus
#define NULL 0L
#else
#define NULL ((void *)0)
#endif

#if __GNUC__ > 3
#define offsetof(type, member) __builtin_offsetof(type, member)
#else
#define offsetof(type, member) ((size_t)((char *)&(((type *)0)->member) - (char *)0))
#endif

#endif // __STDDEF_H__
