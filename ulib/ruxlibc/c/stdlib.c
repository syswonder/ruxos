/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include <ctype.h>
#include <errno.h>
#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

char *program_invocation_short_name = "dummy";
char *program_invocation_name = "dummy";

#define __DECONST(type, var) ((type)(uintptr_t)(const void *)(var))

void srandom(unsigned int s)
{
    srand(s);
}

#ifdef RUX_CONFIG_ALLOC

void *calloc(size_t m, size_t n)
{
    void *mem = malloc(m * n);

    return memset(mem, 0, n * m);
}

void *realloc(void *memblock, size_t size)
{
    if (!memblock)
        return malloc(size);

    size_t o_size = *(size_t *)(memblock - 8);

    void *mem = malloc(size);

    for (int i = 0; i < (o_size < size ? o_size : size); i++)
        ((char *)mem)[i] = ((char *)memblock)[i];

    free(memblock);
    return mem;
}

#endif // RUX_CONFIG_ALLOC

long long llabs(long long a)
{
    return a > 0 ? a : -a;
}

int abs(int a)
{
    return a > 0 ? a : -a;
}

long long atoll(const char *s)
{
    long long n = 0;
    int neg = 0;
    while (isspace(*s)) s++;
    switch (*s) {
    case '-':
        neg = 1;
    case '+':
        s++;
    }
    /* Compute n as a negative number to avoid overflow on LLONG_MIN */
    while (isdigit(*s)) n = 10 * n - (*s++ - '0');
    return neg ? n : -n;
}

long strtol(const char *restrict nptr, char **restrict endptr, int base)
{
    const char *s;
    unsigned long acc;
    unsigned char c;
    unsigned long qbase, cutoff;
    int neg, any, cutlim;

    s = nptr;
    if (base < 0 || base == 1 || base > 36) {
        errno = EINVAL;
        any = 0;
        acc = 0;
        goto exit;
    }

    do {
        c = *s++;
    } while (isspace(c));
    if (c == '-') {
        neg = 1;
        c = *s++;
    } else {
        neg = 0;
        if (c == '+')
            c = *s++;
    }
    if ((base == 0 || base == 16) && c == '0' && (*s == 'x' || *s == 'X')) {
        c = s[1];
        s += 2;
        base = 16;
    }
    if (base == 0)
        base = c == '0' ? 8 : 10;

    qbase = (unsigned int)base;
    cutoff = neg ? (unsigned long)LONG_MAX - (unsigned long)(LONG_MIN + LONG_MAX) : LONG_MAX;
    cutlim = cutoff % qbase;
    cutoff /= qbase;
    for (acc = 0, any = 0;; c = *s++) {
        if (!isascii(c))
            break;
        if (isdigit(c))
            c -= '0';
        else if (isalpha(c))
            c -= isupper(c) ? 'A' - 10 : 'a' - 10;
        else
            break;
        if (c >= base)
            break;
        if (any < 0 || acc > cutoff || (acc == cutoff && c > cutlim))
            any = -1;
        else {
            any = 1;
            acc *= qbase;
            acc += c;
        }
    }

    if (any < 0) {
        acc = neg ? LONG_MIN : LONG_MAX;
        errno = ERANGE;
    } else if (neg)
        acc = -acc;

exit:
    if (endptr != 0)
        *endptr = __DECONST(char *, any ? s - 1 : nptr);
    return acc;
}

unsigned long strtoul(const char *nptr, char **endptr, int base)
{
    const char *s = nptr;
    unsigned long acc;
    unsigned char c;
    unsigned long cutoff;
    int neg = 0, any, cutlim;

    if (base < 0 || base == 1 || base > 36) {
        errno = EINVAL;
        any = 0;
        acc = 0;
        goto exit;
    }

    do {
        c = *s++;
    } while (isspace(c));
    if (c == '-') {
        neg = 1;
        c = *s++;
    } else if (c == '+')
        c = *s++;
    if ((base == 0 || base == 16) && c == '0' && (*s == 'x' || *s == 'X')) {
        c = s[1];
        s += 2;
        base = 16;
    }
    if (base == 0)
        base = c == '0' ? 8 : 10;
    cutoff = (unsigned long)ULONG_MAX / (unsigned long)base;
    cutlim = (unsigned long)ULONG_MAX % (unsigned long)base;

    for (acc = 0, any = 0;; c = *s++) {
        if (!isascii(c))
            break;
        if (isdigit(c))
            c -= '0';
        else if (isalpha(c))
            c -= isupper(c) ? 'A' - 10 : 'a' - 10;
        else
            break;
        if (c >= base)
            break;
        if (any < 0 || acc > cutoff || (acc == cutoff && c > cutlim))
            any = -1;
        else {
            any = 1;
            acc *= base;
            acc += c;
        }
    }
    if (any < 0) {
        acc = ULONG_MAX;
        errno = ERANGE;
    } else if (neg)
        acc = -acc;
exit:
    if (endptr != 0)
        *endptr = __DECONST(char *, any ? s - 1 : nptr);
    return acc;
}

long long strtoll(const char *nptr, char **endptr, int base)
{
    const char *s;
    unsigned long long acc;
    unsigned char c;
    unsigned long long qbase, cutoff;
    int neg, any, cutlim;

    s = nptr;
    if (base < 0 || base == 1 || base > 36) {
        errno = EINVAL;
        any = 0;
        acc = 0;
        goto exit;
    }

    do {
        c = *s++;
    } while (isspace(c));
    if (c == '-') {
        neg = 1;
        c = *s++;
    } else {
        neg = 0;
        if (c == '+')
            c = *s++;
    }
    if ((base == 0 || base == 16) && c == '0' && (*s == 'x' || *s == 'X')) {
        c = s[1];
        s += 2;
        base = 16;
    }
    if (base == 0)
        base = c == '0' ? 8 : 10;

    qbase = (unsigned int)base;
    cutoff = neg ? (unsigned long long)LLONG_MAX - (unsigned long long)(LLONG_MIN + LLONG_MAX)
                 : LLONG_MAX;
    cutlim = cutoff % qbase;
    cutoff /= qbase;
    for (acc = 0, any = 0;; c = *s++) {
        if (!isascii(c))
            break;
        if (isdigit(c))
            c -= '0';
        else if (isalpha(c))
            c -= isupper(c) ? 'A' - 10 : 'a' - 10;
        else
            break;
        if (c >= base)
            break;
        if (any < 0 || acc > cutoff || (acc == cutoff && c > cutlim))
            any = -1;
        else {
            any = 1;
            acc *= qbase;
            acc += c;
        }
    }

    if (any < 0) {
        errno = ERANGE;
        acc = neg ? LLONG_MIN : LLONG_MAX;
    } else if (neg)
        acc = -acc;

exit:
    if (endptr != 0)
        *endptr = __DECONST(char *, any ? s - 1 : nptr);
    return acc;
}

unsigned long long strtoull(const char *nptr, char **endptr, int base)
{
    const char *s = nptr;
    unsigned long long acc;
    unsigned char c;
    unsigned long long qbase, cutoff;
    int neg, any, cutlim;

    if (base < 0 || base == 1 || base > 36) {
        errno = EINVAL;
        any = 0;
        acc = 0;
        goto exit;
    }

    do {
        c = *s++;
    } while (isspace(c));
    if (c == '-') {
        neg = 1;
        c = *s++;
    } else {
        neg = 0;
        if (c == '+')
            c = *s++;
    }
    if ((base == 0 || base == 16) && c == '0' && (*s == 'x' || *s == 'X')) {
        c = s[1];
        s += 2;
        base = 16;
    }
    if (base == 0)
        base = c == '0' ? 8 : 10;

    qbase = (unsigned int)base;
    cutoff = (unsigned long long)ULLONG_MAX / qbase;
    cutlim = (unsigned long long)ULLONG_MAX % qbase;
    for (acc = 0, any = 0;; c = *s++) {
        if (!isascii(c))
            break;
        if (isdigit(c))
            c -= '0';
        else if (isalpha(c))
            c -= isupper(c) ? 'A' - 10 : 'a' - 10;
        else
            break;
        if (c >= base)
            break;
        if (any < 0 || acc > cutoff || (acc == cutoff && c > cutlim))
            any = -1;
        else {
            any = 1;
            acc *= qbase;
            acc += c;
        }
    }
    if (any < 0) {
        errno = ERANGE;
        acc = ULLONG_MAX;
    } else if (neg)
        acc = -acc;

exit:
    if (endptr != 0)
        *endptr = __DECONST(char *, any ? s - 1 : nptr);
    return acc;
}

#ifdef RUX_CONFIG_FP_SIMD

// TODO: precision may not be enough
long double strtold(const char *restrict s, char **restrict p)
{
    return (long double)strtod(s, p);
}

#endif // RUX_CONFIG_FP_SIMD

typedef int (*cmpfun_)(const void *, const void *);

static int wrapper_cmp(const void *v1, const void *v2, void *cmp)
{
    return ((cmpfun_)cmp)(v1, v2);
}

typedef int (*cmpfun)(const void *, const void *, void *);

static inline int a_ctz_32(uint32_t x)
{
    static const char debruijn32[32] = {0,  1,  23, 2,  29, 24, 19, 3,  30, 27, 25,
                                        11, 20, 8,  4,  13, 31, 22, 28, 18, 26, 10,
                                        7,  12, 21, 17, 9,  6,  16, 5,  15, 14};
    return debruijn32[(x & -x) * 0x076be629 >> 27];
}

static inline int a_ctz_64(uint64_t x)
{
    static const char debruijn64[64] = {
        0,  1,  2,  53, 3,  7,  54, 27, 4,  38, 41, 8,  34, 55, 48, 28, 62, 5,  39, 46, 44, 42,
        22, 9,  24, 35, 59, 56, 49, 18, 29, 11, 63, 52, 6,  26, 37, 40, 33, 47, 61, 45, 43, 21,
        23, 58, 17, 10, 51, 25, 36, 32, 60, 20, 57, 16, 50, 31, 19, 15, 30, 14, 13, 12};
    if (sizeof(long) < 8) {
        uint32_t y = x;
        if (!y) {
            y = x >> 32;
            return 32 + a_ctz_32(y);
        }
        return a_ctz_32(y);
    }
    return debruijn64[(x & -x) * 0x022fdd63cc95386dull >> 58];
}

static inline int a_ctz_l(unsigned long x)
{
    return (sizeof(long) < 8) ? a_ctz_32(x) : a_ctz_64(x);
}

#define ntz(x) a_ctz_l((x))

static inline int pntz(size_t p[2])
{
    int r = ntz(p[0] - 1);
    if (r != 0 || (r = 8 * sizeof(size_t) + ntz(p[1])) != 8 * sizeof(size_t)) {
        return r;
    }
    return 0;
}

static void cycle(size_t width, unsigned char *ar[], int n)
{
    unsigned char tmp[256];
    size_t l;
    int i;

    if (n < 2) {
        return;
    }

    ar[n] = tmp;
    while (width) {
        l = sizeof(tmp) < width ? sizeof(tmp) : width;
        memcpy(ar[n], ar[0], l);
        for (i = 0; i < n; i++) {
            memcpy(ar[i], ar[i + 1], l);
            ar[i] += l;
        }
        width -= l;
    }
}

/* shl() and shr() need n > 0 */
static inline void shl(size_t p[2], int n)
{
    if (n >= 8 * sizeof(size_t)) {
        n -= 8 * sizeof(size_t);
        p[1] = p[0];
        p[0] = 0;
    }
    p[1] <<= n;
    p[1] |= p[0] >> (sizeof(size_t) * 8 - n);
    p[0] <<= n;
}

static inline void shr(size_t p[2], int n)
{
    if (n >= 8 * sizeof(size_t)) {
        n -= 8 * sizeof(size_t);
        p[0] = p[1];
        p[1] = 0;
    }
    p[0] >>= n;
    p[0] |= p[1] << (sizeof(size_t) * 8 - n);
    p[1] >>= n;
}

static void sift(unsigned char *head, size_t width, cmpfun cmp, void *arg, int pshift, size_t lp[])
{
    unsigned char *rt, *lf;
    unsigned char *ar[14 * sizeof(size_t) + 1];
    int i = 1;

    ar[0] = head;
    while (pshift > 1) {
        rt = head - width;
        lf = head - width - lp[pshift - 2];

        if (cmp(ar[0], lf, arg) >= 0 && cmp(ar[0], rt, arg) >= 0) {
            break;
        }
        if (cmp(lf, rt, arg) >= 0) {
            ar[i++] = lf;
            head = lf;
            pshift -= 1;
        } else {
            ar[i++] = rt;
            head = rt;
            pshift -= 2;
        }
    }
    cycle(width, ar, i);
}

static void trinkle(unsigned char *head, size_t width, cmpfun cmp, void *arg, size_t pp[2],
                    int pshift, int trusty, size_t lp[])
{
    unsigned char *stepson, *rt, *lf;
    size_t p[2];
    unsigned char *ar[14 * sizeof(size_t) + 1];
    int i = 1;
    int trail;

    p[0] = pp[0];
    p[1] = pp[1];

    ar[0] = head;
    while (p[0] != 1 || p[1] != 0) {
        stepson = head - lp[pshift];
        if (cmp(stepson, ar[0], arg) <= 0) {
            break;
        }
        if (!trusty && pshift > 1) {
            rt = head - width;
            lf = head - width - lp[pshift - 2];
            if (cmp(rt, stepson, arg) >= 0 || cmp(lf, stepson, arg) >= 0) {
                break;
            }
        }

        ar[i++] = stepson;
        head = stepson;
        trail = pntz(p);
        shr(p, trail);
        pshift += trail;
        trusty = 0;
    }
    if (!trusty) {
        cycle(width, ar, i);
        sift(head, width, cmp, arg, pshift, lp);
    }
}

void __qsort_r(void *base, size_t nel, size_t width, cmpfun cmp, void *arg)
{
    size_t lp[12 * sizeof(size_t)];
    size_t i, size = width * nel;
    unsigned char *head, *high;
    size_t p[2] = {1, 0};
    int pshift = 1;
    int trail;

    if (!size)
        return;

    head = base;
    high = head + size - width;

    /* Precompute Leonardo numbers, scaled by element width */
    for (lp[0] = lp[1] = width, i = 2; (lp[i] = lp[i - 2] + lp[i - 1] + width) < size; i++)
        ;

    while (head < high) {
        if ((p[0] & 3) == 3) {
            sift(head, width, cmp, arg, pshift, lp);
            shr(p, 2);
            pshift += 2;
        } else {
            if (lp[pshift - 1] >= high - head) {
                trinkle(head, width, cmp, arg, p, pshift, 0, lp);
            } else {
                sift(head, width, cmp, arg, pshift, lp);
            }

            if (pshift == 1) {
                shl(p, 1);
                pshift = 0;
            } else {
                shl(p, pshift - 1);
                pshift = 1;
            }
        }

        p[0] |= 1;
        head += width;
    }

    trinkle(head, width, cmp, arg, p, pshift, 0, lp);

    while (pshift != 1 || p[0] != 1 || p[1] != 0) {
        if (pshift <= 1) {
            trail = pntz(p);
            shr(p, trail);
            pshift += trail;
        } else {
            shl(p, 2);
            pshift -= 2;
            p[0] ^= 7;
            shr(p, 1);
            trinkle(head - lp[pshift] - width, width, cmp, arg, p, pshift + 1, 1, lp);
            shl(p, 1);
            p[0] |= 1;
            trinkle(head - width, width, cmp, arg, p, pshift, 1, lp);
        }
        head -= width;
    }
}

void qsort(void *base, size_t nel, size_t width, cmpfun_ cmp)
{
    __qsort_r(base, nel, width, wrapper_cmp, (void *)cmp);
}

// TODO
int mkstemp(char *__template)
{
    unimplemented();
    return 0;
}

// TODO
int mkostemp(char *__template, int __flags)
{
    unimplemented();
    return 0;
}

// TODO
int system(const char *cmd)
{
    unimplemented();
    return 0;
}

// TODO
char *realpath(const char *restrict path, char *restrict resolved_path)
{
    unimplemented();
    return 0;
}

struct chunk {
    size_t psize, csize;
    struct chunk *next, *prev;
};

void __bin_chunk(struct chunk *)
{
    unimplemented();
    return;
}

void *aligned_alloc(size_t align, size_t len)
{
    unsigned char *mem, *new;

    if ((align & -align) != align) {
        errno = EINVAL;
        return 0;
    }

    if (len > SIZE_MAX - align) {
        errno = ENOMEM;
        return 0;
    }

    if (align <= SIZE_ALIGN)
        return malloc(len);

    if (!(mem = malloc(len + align - 1)))
        return 0;

    new = (void *)((uintptr_t)mem + align - 1 & -align);
    if (new == mem)
        return mem;

    struct chunk *c = MEM_TO_CHUNK(mem);
    struct chunk *n = MEM_TO_CHUNK(new);

    if (IS_MMAPPED(c)) {
        n->psize = c->psize + (new - mem);
        n->csize = c->csize - (new - mem);
        return new;
    }

    struct chunk *t = NEXT_CHUNK(c);

    n->psize = c->csize = C_INUSE | (new - mem);
    n->csize = t->psize -= new - mem;

    __bin_chunk(c);
    return new;
}

// TODO
int posix_memalign(void **res, size_t align, size_t len)
{
    if (align < sizeof(void *))
        return EINVAL;
    void *mem = aligned_alloc(align, len);
    if (!mem)
        return errno;
    *res = mem;
    return 0;
}
