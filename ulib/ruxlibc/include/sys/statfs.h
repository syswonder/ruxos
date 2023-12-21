#ifndef _SYS_STATFS_H
#define _SYS_STATFS_H

#include <features.h>
#include <sys/statvfs.h>

typedef struct __fsid_t {
    int __val[2];
} fsid_t;

struct statfs {
    unsigned long f_type, f_bsize;
    fsblkcnt_t f_blocks, f_bfree, f_bavail;
    fsfilcnt_t f_files, f_ffree;
    fsid_t f_fsid;
    unsigned long f_namelen, f_frsize, f_flags, f_spare[4];
};

int statfs(const char *, struct statfs *);
int fstatfs(int, struct statfs *);

#endif
