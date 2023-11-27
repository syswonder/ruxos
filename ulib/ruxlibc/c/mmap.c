/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stddef.h>
#include <stdio.h>
#include <sys/mman.h>

// TODO:
void *mmap(void *addr, size_t len, int prot, int flags, int fildes, off_t off)
{
    unimplemented();
    return MAP_FAILED;
}

// TODO:
int munmap(void *addr, size_t length)
{
    unimplemented();
    return 0;
}

// TODO:
void *mremap(void *old_address, size_t old_size, size_t new_size, int flags,
             ... /* void *new_address */)
{
    unimplemented();
    return NULL;
}

// TODO
int mprotect(void *addr, size_t len, int prot)
{
    unimplemented();
    return 0;
}

// TODO
int madvise(void *addr, size_t len, int advice)
{
    unimplemented();
    return 0;
}
