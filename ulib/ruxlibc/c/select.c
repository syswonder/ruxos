/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifdef RUX_CONFIG_SELECT

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/select.h>
#include <sys/time.h>

int pselect(int n, fd_set *restrict rfds, fd_set *restrict wfds, fd_set *restrict efds,
            const struct timespec *restrict ts, const sigset_t *restrict mask)
{
    struct timeval tv = {ts->tv_sec, ts->tv_nsec / 1000};
    select(n, rfds, wfds, efds, &tv);
    return 0;
}

#endif // RUX_CONFIG_SELECT
