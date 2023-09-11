#include <unistd.h>
#include <sys/mman.h>
#include <errno.h>
#include <stdint.h>
#include <limits.h>
#include "syscall.h"
// #include <stdio.h>
static void dummy(void) { }
weak_alias(dummy, __vm_wait);

#define UNIT SYSCALL_MMAP2_UNIT
#define OFF_MASK ((-0x2000ULL << (8*sizeof(syscall_arg_t)-1)) | (UNIT-1))

void *__mmap(void *start, size_t len, int prot, int flags, int fd, off_t off)
{
	// printf("into mmap 1\n");
	long ret;
	if (off & OFF_MASK) {
		// printf("into mmap 2\n");
		errno = EINVAL;
		return MAP_FAILED;
	}
	if (len >= PTRDIFF_MAX) {
		// printf("into mmap 3\n");
		errno = ENOMEM;
		return MAP_FAILED;
	}
	if (flags & MAP_FIXED) {
		// printf("into mmap 4\n");
		__vm_wait();
	}
#ifdef SYS_mmap2
	ret = __syscall(SYS_mmap2, start, len, prot, flags, fd, off/UNIT);
#else
	// printf("into mmap 5\n");
	ret = __syscall(SYS_mmap, start, len, prot, flags, fd, off);
	// printf("into mmap 6, ret = %d\n", ret);
#endif
	/* Fixup incorrect EPERM from kernel. */
	if (ret == -EPERM && !start && (flags&MAP_ANON) && !(flags&MAP_FIXED)){
		// printf("into mmap 7\n");
		ret = -ENOMEM;
		// printf("after seteno in mmap\n");
	}
		
	return (void *)__syscall_ret(ret);
}

weak_alias(__mmap, mmap);

weak_alias(mmap, mmap64);
