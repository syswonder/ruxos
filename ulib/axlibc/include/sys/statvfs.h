#ifndef _SYS_STATVFS_H
#define _SYS_STATVFS_H

#include <features.h>
#include <stddef.h>

struct statvfs {
    unsigned long f_bsize, f_frsize;
    fsblkcnt_t f_blocks, f_bfree, f_bavail;
    fsfilcnt_t f_files, f_ffree, f_favail;
#if __BYTE_ORDER == __LITTLE_ENDIAN
    unsigned long f_fsid;
    unsigned : 8 * (2 * sizeof(int) - sizeof(long));
#else
    unsigned : 8 * (2 * sizeof(int) - sizeof(long));
    unsigned long f_fsid;
#endif
    unsigned long f_flag, f_namemax;
    int __reserved[6];
};

int statvfs(const char *__restrict, struct statvfs *__restrict);
int fstatvfs(int, struct statvfs *);

#endif
