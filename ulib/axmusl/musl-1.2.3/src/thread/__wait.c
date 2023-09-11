#include "pthread_impl.h"
#include <stdio.h>

void __wait(volatile int *addr, volatile int *waiters, int val, int priv)
{
	// printf("into __wait, addr = %p, val: %d, waiters = %d\n", addr, val, *waiters);
	int spins=100;
	if (priv) priv = FUTEX_PRIVATE;
	while (spins-- && (!waiters || !*waiters)) {
		if (*addr==val) a_spin();
		else return;
	}
	if (waiters) a_inc(waiters);
	// printf("before while, into __wait, addr = %p, val: %d\n", addr, val);
	while (*addr==val) {
		// printf("into while, into __wait, addr = %p, val: %d\n", addr, val);
		__syscall(SYS_futex, addr, FUTEX_WAIT|priv, val, 0) != -ENOSYS
		|| __syscall(SYS_futex, addr, FUTEX_WAIT, val, 0);
	}
	if (waiters) a_dec(waiters);
}
