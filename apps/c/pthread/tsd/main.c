/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include <assert.h>
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

static pthread_key_t p_key;

void *specific_func(void *arg)
{
    int *p = (int *)malloc(sizeof(int));
    *p = *(int *)arg;
    pthread_setspecific(p_key, p);
    if (*p == 0x5678) {
        sleep(1);
    }
    int *tmp = (int *)pthread_getspecific(p_key);
    assert(*tmp == *(int *)arg);
    assert(pthread_getspecific(999999) == NULL);
    return NULL;
}

int res = 0;

void destr_func(void *arg)
{
    res += *(int *)arg;
    char *buf[100];
    sprintf(buf, "destr_func, *arg = 0x%x", *(int *)arg);
    puts(buf);
    free(arg);
}

void test_specific()
{
    int max_keys = sysconf(_SC_THREAD_KEYS_MAX);
    pthread_key_create(&p_key, destr_func);
    printf("max_keys = %d, got No.%d\n", max_keys, p_key);

    pthread_t t1, t2;
    int arg1 = 0x1234, arg2 = 0x5678;
    pthread_create(&t1, NULL, specific_func, &arg1);
    pthread_create(&t2, NULL, specific_func, &arg2);
    pthread_join(t1, NULL);
    pthread_join(t2, NULL);
    if (res != 0x1234 + 0x5678) {
        puts("TSD test fail");
    } else {
        puts("TSD test success");
    }

    pthread_key_delete(p_key);
}

int main()
{
    pthread_t main_thread = pthread_self();
    assert(main_thread != 0);

    test_specific();

    puts("(C)Pthread TSD tests run OK!");

    return 0;
}
