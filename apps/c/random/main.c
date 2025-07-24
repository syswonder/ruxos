/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#define _GNU_SOURCE
#include <sys/random.h>
#include <stdio.h>

int main() {
    printf("Using getrandom to fetch random bytes...\n");
    unsigned char buf[16];
    ssize_t n = getrandom(buf, sizeof(buf), 0);
    if (n < 0) {
        perror("getrandom");
        return 1;
    }

    printf("Random bytes:\n");
    for (int i = 0; i < n; ++i) {
        printf("%02x ", buf[i]);
    }
    printf("\n");

    return 0;
}
