/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef __TIME_H__
#define __TIME_H__

#include <stddef.h>
#include <sys/time.h>

#define CLOCK_REALTIME  0
#define CLOCK_MONOTONIC 1
#define CLOCKS_PER_SEC  1000000L

#define __tm_gmtoff tm_gmtoff

struct tm {
    int tm_sec;   /* seconds of minute */
    int tm_min;   /* minutes of hour */
    int tm_hour;  /* hours of day */
    int tm_mday;  /* day of month */
    int tm_mon;   /* month of year, 0 is first month(January) */
    int tm_year;  /* years, whose value equals the actual year minus 1900 */
    int tm_wday;  /* day of week, 0 is sunday, 1 is monday, and so on */
    int tm_yday;  /* day of year */
    int tm_isdst; /* daylight saving time flag */
    long int __tm_gmtoff;
    const char *__tm_zone;
};

clock_t clock(void);
time_t time(time_t *);
double difftime(time_t, time_t);
time_t mktime(struct tm *);
size_t strftime(char *__restrict, size_t, const char *__restrict, const struct tm *__restrict);
struct tm *gmtime(const time_t *);
struct tm *localtime(const time_t *);

struct tm *gmtime_r(const time_t *__restrict, struct tm *__restrict);
struct tm *localtime_r(const time_t *__restrict, struct tm *__restrict);
char *asctime_r(const struct tm *__restrict, char *__restrict);
char *ctime_r(const time_t *, char *);

void tzset(void);

int nanosleep(const struct timespec *requested_time, struct timespec *remaining);
int clock_gettime(clockid_t _clk, struct timespec *ts);
int clock_settime(clockid_t, const struct timespec *);

#endif // __TIME_H__
