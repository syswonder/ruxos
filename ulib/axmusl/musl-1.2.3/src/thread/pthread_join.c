#define _GNU_SOURCE
#include "pthread_impl.h"
#include <sys/mman.h>
// #include <stdio.h>

static void dummy1(pthread_t t)
{
}
weak_alias(dummy1, __tl_sync);

static int __pthread_timedjoin_np(pthread_t t, void **res, const struct timespec *at)
{
	int state, cs, r = 0;
	__pthread_testcancel();
	__pthread_setcancelstate(PTHREAD_CANCEL_DISABLE, &cs);
	if (cs == PTHREAD_CANCEL_ENABLE) __pthread_setcancelstate(cs, 0);
	// printf("before while t->state = %d, addr: %p\n", t->detach_state, &t->detach_state);
	while ((state = t->detach_state) && r != ETIMEDOUT && r != EINVAL) {
		// printf("into while t->state = %d, addr: %p, state = %d\n", t->detach_state, &t->detach_state, state);
		if (state >= DT_DETACHED) a_crash();
		r = __timedwait_cp(&t->detach_state, state, CLOCK_REALTIME, at, 1);
		// printf("-----------------r = %d, into while t->state = %d, addr: %p, state = %d\n", r, t->detach_state, &t->detach_state, state);
	}
	// printf("quit while\n");
	__pthread_setcancelstate(cs, 0);
	if (r == ETIMEDOUT || r == EINVAL) return r;
	// printf("before tl_sync\n");
	__tl_sync(t);
	// printf("after __tl_sync\n");
	if (res) *res = t->result;
	// printf("before munmap\n");
	if (t->map_base) __munmap(t->map_base, t->map_size);
	return 0;
}

int __pthread_join(pthread_t t, void **res)
{
	return __pthread_timedjoin_np(t, res, 0);
}

static int __pthread_tryjoin_np(pthread_t t, void **res)
{
	return t->detach_state==DT_JOINABLE ? EBUSY : __pthread_join(t, res);
}

weak_alias(__pthread_tryjoin_np, pthread_tryjoin_np);
weak_alias(__pthread_timedjoin_np, pthread_timedjoin_np);
weak_alias(__pthread_join, pthread_join);
