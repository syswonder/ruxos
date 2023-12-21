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
#include <fcntl.h>
#include <stdio.h>
#include <stdlib.h>
#include <sys/types.h>
#include <sys/utsname.h>
#include <time.h>
#include <unistd.h>

// TODO:
pid_t getppid(void)
{
    unimplemented();
    return 0;
}

// TODO:
uid_t geteuid(void)
{
    unimplemented();
    return 0;
}

// TODO
uid_t getuid(void)
{
    unimplemented();
    return 0;
}

// TODO
int setuid(uid_t __uid)
{
    unimplemented();
    return 0;
}

// TODO
pid_t setsid(void)
{
    unimplemented();
    return 0;
}

// TODO
int setgid(gid_t gid)
{
    unimplemented();
    return 0;
}

// TODO
int isatty(int fd)
{
    unimplemented();
    return 0;
}

// TODO
int getpagesize(void)
{
    unimplemented();
    return 0;
}

// TODO
ssize_t pread(int fd, void *buf, size_t count, off_t offset)
{
    unimplemented();
    return 0;
}

// TODO
ssize_t pwrite(int fd, const void *buf, size_t count, off_t offset)
{
    unimplemented();
    return 0;
}

// TODO
int gethostname(char *name, size_t len)
{
    size_t i;
    struct utsname uts;
    if (uname(&uts))
        return -1;
    if (len > sizeof uts.nodename)
        len = sizeof uts.nodename;
    for (i = 0; i < len && (name[i] = uts.nodename[i]); i++)
        ;
    if (i && i == len)
        name[i - 1] = 0;
    return 0;
}

// TODO
int chown(const char *path, uid_t owner, gid_t group)
{
    unimplemented();
    return 0;
}

unsigned int sleep(unsigned int seconds)
{
    struct timespec ts;

    ts.tv_sec = seconds;
    ts.tv_nsec = 0;
    if (nanosleep(&ts, &ts))
        return ts.tv_sec;

    return 0;
}

int usleep(unsigned useconds)
{
    struct timespec tv = {.tv_sec = useconds / 1000000, .tv_nsec = (useconds % 1000000) * 1000};
    return nanosleep(&tv, &tv);
}

#ifdef RUX_CONFIG_FS

// TODO:
int access(const char *pathname, int mode)
{
    unimplemented();
    return 0;
}

// TODO:
ssize_t readlink(const char *path, char *buf, size_t bufsiz)
{
    unimplemented();
    return 0;
}

// TODO:
int fsync(int fd)
{
    unimplemented();
    return 0;
}

// TODO
int fdatasync(int __fildes)
{
    unimplemented();
    return 0;
}

// TODO:
int fchown(int fd, uid_t owner, gid_t group)
{
    unimplemented("owner: %x group: %x", owner, group);
    return 0;
}

// TODO:
int ftruncate(int fd, off_t length)
{
    unimplemented();
    return 0;
}

// TODO
int chdir(const char *__path)
{
    unimplemented();
    return 0;
}

// TODO
int truncate(const char *path, off_t length)
{
    unimplemented();
    return 0;
}

#endif // RUX_CONFIG_FS

#ifdef RUX_CONFIG_PIPE

int pipe2(int fd[2], int flag)
{
    if (!flag)
        return pipe(fd);
    if (flag & ~(O_CLOEXEC | O_NONBLOCK))
        return -EINVAL;

    int res = pipe(fd);
    if (res != 0)
        return res;

    if (flag & O_CLOEXEC) {
        fcntl(fd[0], F_SETFD, FD_CLOEXEC);
        fcntl(fd[1], F_SETFD, FD_CLOEXEC);
    }
    if (flag & O_NONBLOCK) {
        fcntl(fd[0], F_SETFL, O_NONBLOCK);
        fcntl(fd[1], F_SETFL, O_NONBLOCK);
    }

    return 0;
}

#endif // RUX_CONFIG_PIPE

// TODO
_Noreturn void _exit(int status)
{
    exit(status);
}

// TODO
int execve(const char *__path, char *const *__argv, char *const *__envp)
{
    unimplemented();
    return 0;
}

// TODO
pid_t fork(void)
{
    unimplemented();
    return -1;
}
