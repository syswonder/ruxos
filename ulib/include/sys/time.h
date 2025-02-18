/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef __SYS_TIME_H__
#define __SYS_TIME_H__

#include <stdint.h>

#define ITIMER_REAL    0
#define ITIMER_VIRTUAL 1
#define ITIMER_PROF    2

extern long timezone;
typedef long long time_t;

struct timeval {
    time_t tv_sec; /* seconds */
    long tv_usec;  /* microseconds */
};

struct timespec {
    time_t tv_sec; /* seconds */
    long tv_nsec;  /* nanoseconds */
};

typedef struct timespec timespec;

struct timezone {
    int tz_minuteswest; /* (minutes west of Greenwich) */
    int tz_dsttime;     /* (type of DST correction) */
};

struct itimerval {
    struct timeval it_interval;
    struct timeval it_value;
};

int gettimeofday(struct timeval *tv, struct timezone *tz);
int settimeofday(const struct timeval *, const struct timezone *);

int getitimer(int, struct itimerval *);
int setitimer(int, const struct itimerval *__restrict, struct itimerval *__restrict);
int utimes(const char *filename, const struct timeval times[2]);

#endif
