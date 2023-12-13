/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

int main(int argc, char *argv[])
{
    puts("Running memory tests...");
    uintptr_t *brk = (uintptr_t *)malloc(0);
    if (brk != NULL)
        printf("top of heap=%p\n", brk);
    else
        printf("allocation fail\n");

    int n = 9;
    int i = 0;
    uintptr_t **p = (uintptr_t **)malloc(n * sizeof(uint64_t));
    if (p != NULL)
        printf("%d(+8)Byte allocated: p=%p\n", n * sizeof(uint64_t), p);
    else {
        printf("malloc fail\n");
        return -1;
    }
    printf("allocate %d(+8)Byte for %d times:\n", sizeof(uint64_t), n);
    for (i = 0; i < n; i++) {
        p[i] = (uintptr_t *)malloc(sizeof(uint64_t));
        *p[i] = 233;
        printf("allocated addr=%p\n", p[i]);
    }
    for (i = 0; i < n; i++) {
        free(p[i]);
    }
    free(p);
    puts("Memory tests run OK!");
    return 0;
}
