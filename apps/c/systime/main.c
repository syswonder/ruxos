/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stdio.h>
#include <sys/time.h>

int main()
{
    struct timeval tv;
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);

    usleep(3000000);

    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);

    struct timeval new_time;
    new_time.tv_sec = 1731110400;
    new_time.tv_usec = 0;

    if (settimeofday(&new_time, NULL) != 0 ) {
        perror("settimeofday");
        return -1;
    }
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return -1;
    }

    printf("now time: %ld : %ld\n", tv.tv_sec,tv.tv_usec);
    return 0;

}