/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifdef RUX_CONFIG_MULTITASK

#include <errno.h>
#include <limits.h>
#include <pthread.h>
#include <stdio.h>
#include <unistd.h>

int pthread_setcancelstate(int new, int *old)
{
    unimplemented();
    return 0;
}

int pthread_setcanceltype(int new, int *old)
{
    unimplemented();
    return 0;
}

// TODO
void pthread_testcancel(void)
{
    unimplemented();
    return;
}

// TODO
int pthread_cancel(pthread_t t)
{
    unimplemented();
    return 0;
}

// TODO
int pthread_setname_np(pthread_t thread, const char *name)
{
    unimplemented();
    return 0;
}

#define DEFAULT_STACK_SIZE 131072
#define DEFAULT_GUARD_SIZE 8192

// TODO
int pthread_attr_init(pthread_attr_t *a)
{
    *a = (pthread_attr_t){0};
    // __acquire_ptc();
    a->_a_stacksize = DEFAULT_STACK_SIZE;
    a->_a_guardsize = DEFAULT_GUARD_SIZE;
    // __release_ptc();
    return 0;
}

int pthread_attr_getstacksize(const pthread_attr_t *restrict a, size_t *restrict size)
{
    *size = a->_a_stacksize;
    return 0;
}

int pthread_attr_setstacksize(pthread_attr_t *a, size_t size)
{
    if (size - PTHREAD_STACK_MIN > SIZE_MAX / 4)
        return EINVAL;
    a->_a_stackaddr = 0;
    a->_a_stacksize = size;
    return 0;
}

#endif // RUX_CONFIG_MULTITASK
