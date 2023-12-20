/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef __STDLIB_H__
#define __STDLIB_H__

#include <features.h>
#include <stddef.h>

#define RAND_MAX (0x7fffffff)

#define WEXITSTATUS(s) (((s)&0xff00) >> 8)
#define WTERMSIG(s)    ((s)&0x7f)
#define WIFEXITED(s)   (!WTERMSIG(s))
#define WIFSIGNALED(s) (((s)&0xffff) - 1U < 0xffu)

#define EXIT_FAILURE 1
#define EXIT_SUCCESS 0

#define SIZE_ALIGN (4*sizeof(size_t))
#define SIZE_MASK (-SIZE_ALIGN)
#define OVERHEAD (2*sizeof(size_t))
#define MMAP_THRESHOLD (0x1c00*SIZE_ALIGN)
#define DONTCARE 16
#define RECLAIM 163840

#define CHUNK_SIZE(c) ((c)->csize & -2)
#define CHUNK_PSIZE(c) ((c)->psize & -2)
#define PREV_CHUNK(c) ((struct chunk *)((char *)(c) - CHUNK_PSIZE(c)))
#define NEXT_CHUNK(c) ((struct chunk *)((char *)(c) + CHUNK_SIZE(c)))
#define MEM_TO_CHUNK(p) (struct chunk *)((char *)(p) - OVERHEAD)
#define CHUNK_TO_MEM(c) (void *)((char *)(c) + OVERHEAD)
#define BIN_TO_CHUNK(i) (MEM_TO_CHUNK(&mal.bins[i].head))

#define C_INUSE  ((size_t)1)

#define IS_MMAPPED(c) !((c)->csize & (C_INUSE))

int posix_memalign (void **, size_t, size_t);
char *realpath (const char *__restrict, char *__restrict);

long long atoll(const char *nptr);

float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);

long strtol(const char *__restrict, char **__restrict, int);
unsigned long strtoul(const char *nptr, char **endptr, int base);
long long strtoll(const char *nptr, char **endptr, int base);
unsigned long long strtoull(const char *nptr, char **endptr, int base);

int rand(void);
void srand(unsigned);
long random(void);
void srandom(unsigned int);

#ifdef RUX_CONFIG_FP_SIMD
float strtof(const char *__restrict, char **__restrict);
double strtod(const char *__restrict, char **__restrict);
long double strtold(const char *__restrict, char **__restrict);
#endif

void qsort(void *, size_t, size_t, int (*)(const void *, const void *));

void *malloc(size_t);
void *calloc(size_t, size_t);
void *realloc(void *, size_t);
void free(void *);

_Noreturn void abort(void);
_Noreturn void exit(int);

char *getenv(const char *);

int abs(int);
long labs(long);
long long llabs(long long);

int mkstemp(char *);
int mkostemp(char *, int);
int setenv(const char *, const char *, int);
int unsetenv(const char *);
int system(const char *);

#endif //__STDLIB_H__
