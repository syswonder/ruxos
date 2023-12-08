#ifndef _MALLOC_H
#define _MALLOC_H


#define __NEED_size_t

#include <bits/alltypes.h>

void *valloc (size_t);
void *memalign(size_t, size_t);

size_t malloc_usable_size(void *);


#endif