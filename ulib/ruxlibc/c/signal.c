/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include <errno.h>
#include <signal.h>
#include <stddef.h>
#include <stdio.h>

extern int sigaction_inner(int, const struct sigaction *, struct sigaction *);

void (*signal(int signum, void (*handler)(int)))(int)
{
    struct sigaction old;
    struct sigaction act = {
        .sa_handler = handler, .sa_flags = SA_RESTART, /* BSD signal semantics */
    };

    if (sigaction_inner(signum, &act, &old) < 0)
        return SIG_ERR;

    return (old.sa_flags & SA_SIGINFO) ? NULL : old.sa_handler;
}

int sigaction(int sig, const struct sigaction *restrict act, struct sigaction *restrict oact)
{
    return sigaction_inner(sig, act, oact);
}

// TODO
int kill(pid_t __pid, int __sig)
{
    unimplemented();
    return 0;
}

int sigemptyset(sigset_t *set)
{
    set->__bits[0] = 0;
    if (sizeof(long) == 4 || _NSIG > 65)
        set->__bits[1] = 0;
    if (sizeof(long) == 4 && _NSIG > 65) {
        set->__bits[2] = 0;
        set->__bits[3] = 0;
    }
    return 0;
}

// TODO
int raise(int __sig)
{
    unimplemented();
    return 0;
}

int sigaddset(sigset_t *set, int sig)
{
    unsigned s = sig - 1;
    if (s >= _NSIG - 1 || sig - 32U < 3) {
        errno = EINVAL;
        return -1;
    }
    set->__bits[s / 8 / sizeof *set->__bits] |= 1UL << (s & (8 * sizeof *set->__bits - 1));
    return 0;
}

// TODO
int pthread_sigmask(int __how, const sigset_t *restrict __newmask, sigset_t *restrict __oldmask)
{
    unimplemented();
    return 0;
}

// TODO
int sigprocmask(int how, const sigset_t *__restrict set, sigset_t *__restrict oldset)
{
    unimplemented();
    return 0;
}

// TODO
int sigsuspend(const sigset_t *mask)
{
    unimplemented();
    return 0;
}

#ifdef RUX_CONFIG_MULTITASK
// TODO
int pthread_kill(pthread_t t, int sig)
{
    unimplemented();
    return 0;
}
#endif
