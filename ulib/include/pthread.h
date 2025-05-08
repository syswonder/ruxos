/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef _PTHREAD_H
#define _PTHREAD_H

#include <features.h>

typedef void *pthread_t;
#include <time.h>

#define PTHREAD_CANCEL_ENABLE  0
#define PTHREAD_CANCEL_DISABLE 1
#define PTHREAD_CANCEL_MASKED  2

#define PTHREAD_CANCEL_DEFERRED     0
#define PTHREAD_CANCEL_ASYNCHRONOUS 1

typedef struct {
    unsigned __attr;
} pthread_condattr_t;

#include <ax_pthread_cond.h>
#include <ax_pthread_mutex.h>
typedef struct {
    unsigned __attr;
} pthread_mutexattr_t;

typedef struct {
    union {
        int __i[sizeof(long) == 8 ? 14 : 9];
        volatile int __vi[sizeof(long) == 8 ? 14 : 9];
        unsigned long __s[sizeof(long) == 8 ? 7 : 9];
    } __u;
} pthread_attr_t;
#define _a_stacksize __u.__s[0]
#define _a_guardsize __u.__s[1]
#define _a_stackaddr __u.__s[2]


#define PTHREAD_CANCELED ((void *)-1)
#define SIGCANCEL        33

/* Keys for thread-specific data */
typedef unsigned int pthread_key_t;

#ifdef RUX_CONFIG_MULTITASK

_Noreturn void pthread_exit(void *);
pthread_t pthread_self(void);

int pthread_create(pthread_t *__restrict, const pthread_attr_t *__restrict, void *(*)(void *),
                   void *__restrict);
int pthread_join(pthread_t t, void **res);

int pthread_setcancelstate(int, int *);
int pthread_setcanceltype(int, int *);
void pthread_testcancel(void);
int pthread_cancel(pthread_t);

int pthread_mutex_init(pthread_mutex_t *__restrict, const pthread_mutexattr_t *__restrict);
int pthread_mutex_destroy(pthread_mutex_t *);
int pthread_mutex_lock(pthread_mutex_t *);
int pthread_mutex_unlock(pthread_mutex_t *);
int pthread_mutex_trylock(pthread_mutex_t *);

int pthread_setname_np(pthread_t, const char *);

int pthread_cond_init(pthread_cond_t *__restrict__ __cond,
                      const pthread_condattr_t *__restrict__ __cond_attr);
int pthread_cond_destroy(pthread_cond_t *__cond);
int pthread_cond_signal(pthread_cond_t *__cond);
int pthread_cond_timedwait(pthread_cond_t *__restrict__ __cond,
                           pthread_mutex_t *__restrict__ __mutex,
                           const struct timespec *__restrict__ __abstime);
int pthread_cond_wait(pthread_cond_t *__restrict__ __cond, pthread_mutex_t *__restrict__ __mutex);
int pthread_cond_broadcast(pthread_cond_t *);

int pthread_attr_init(pthread_attr_t *__attr);
int pthread_attr_getstacksize(const pthread_attr_t *__restrict__ __attr,
                              size_t *__restrict__ __stacksize);
int pthread_attr_setstacksize(pthread_attr_t *__attr, size_t __stacksize);

/* Create a key value identifying a location in the thread-specific
   data area.  Each thread maintains a distinct thread-specific data
   area.  DESTR_FUNCTION, if non-NULL, is called with the value
   associated to that key when the key is destroyed.
   DESTR_FUNCTION is not called if the value associated is NULL when
   the key is destroyed.  */
int pthread_key_create(pthread_key_t *__key, void (*__destr_function)(void *));

/* Destroy KEY.  */
int pthread_key_delete(pthread_key_t __key);

/* Return current value of the thread-specific data slot identified by KEY.  */
void *pthread_getspecific(pthread_key_t __key);

/* Store POINTER in the thread-specific data slot identified by KEY. */
int pthread_setspecific(pthread_key_t __key, const void *__pointer);

#endif // RUX_CONFIG_MULTITASK

#endif // _PTHREAD_H
