/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include "printf.h"
#include <assert.h>
#include <errno.h>
#include <fcntl.h>
#include <limits.h>
#include <stdarg.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>

// LOCK used by `puts()`
#ifdef RUX_CONFIG_MULTITASK
#include <pthread.h>
static pthread_mutex_t lock = PTHREAD_MUTEX_INITIALIZER;
#endif

#define MAX(a, b) ((a) > (b) ? (a) : (b))
#define MIN(a, b) ((a) < (b) ? (a) : (b))

FILE __stdin_FILE = {.fd = 0, .buffer_len = 0};

FILE __stdout_FILE = {.fd = 1, .buffer_len = 0};

FILE __stderr_FILE = {.fd = 2, .buffer_len = 0};

FILE *const stdin = &__stdin_FILE;
FILE *const stdout = &__stdout_FILE;
FILE *const stderr = &__stderr_FILE;

// Returns: number of chars written, negative for failure
// Warn: buffer_len[f] will not be changed
static int __write_buffer(FILE *f)
{
    int r = 0;
    if (f->buffer_len == 0)
        return 0;
    r = write(f->fd, f->buf, f->buffer_len);
    return r;
}

// Clear buffer_len[f]
static void __clear_buffer(FILE *f)
{
    f->buffer_len = 0;
}

static int __fflush(FILE *f)
{
    int r = __write_buffer(f);
    __clear_buffer(f);
    return r >= 0 ? 0 : r;
}

static int out(FILE *f, const char *s, size_t l)
{
    int ret = 0;
    for (size_t i = 0; i < l; i++) {
        char c = s[i];
        f->buf[f->buffer_len++] = c;
        if (f->buffer_len == FILE_BUF_SIZE || c == '\n') {
            int r = __write_buffer(f);
            __clear_buffer(f);
            if (r < 0)
                return r;
            if (r < f->buffer_len)
                return ret + r;
            ret += r;
        }
    }
    return ret;
}

int getchar(void)
{
    unimplemented();
    return 0;
}

int fflush(FILE *f)
{
    return __fflush(f);
}

static inline int do_putc(int c, FILE *f)
{
    char byte = c;
    return out(f, &byte, 1);
}

int fputc(int c, FILE *f)
{
    return do_putc(c, f);
}

int putc(int c, FILE *f)
{
    return do_putc(c, f);
}

int putchar(int c)
{
    return do_putc(c, stdout);
}

int puts(const char *s)
{
#ifdef RUX_CONFIG_MULTITASK
    pthread_mutex_lock(&lock);
#endif

    int r = write(1, (const void *)s, strlen(s));
    char brk[1] = {'\n'};
    write(1, (const void *)brk, 1);

#ifdef RUX_CONFIG_MULTITASK
    pthread_mutex_unlock(&lock);
#endif

    return r;
}

void perror(const char *msg)
{
    FILE *f = stderr;
    char *errstr = strerror(errno);

    if (msg && *msg) {
        out(f, msg, strlen(msg));
        out(f, ": ", 2);
    }
    out(f, errstr, strlen(errstr));
    out(f, "\n", 1);
}

static void __out_wrapper(char c, void *arg)
{
    out(arg, &c, 1);
}

int printf(const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfprintf(stdout, fmt, ap);
    va_end(ap);
    return ret;
}

int fprintf(FILE *restrict f, const char *restrict fmt, ...)
{
    int ret;
    va_list ap;
    va_start(ap, fmt);
    ret = vfprintf(f, fmt, ap);
    va_end(ap);
    return ret;
}

int vfprintf(FILE *restrict f, const char *restrict fmt, va_list ap)
{
    return vfctprintf(__out_wrapper, f, fmt, ap);
}

// TODO
int sscanf(const char *restrict __s, const char *restrict __format, ...)
{
    unimplemented();
    return 0;
}

#ifdef RUX_CONFIG_FS

int __fmodeflags(const char *mode)
{
    int flags;
    if (strchr(mode, '+'))
        flags = O_RDWR;
    else if (*mode == 'r')
        flags = O_RDONLY;
    else
        flags = O_WRONLY;
    if (strchr(mode, 'x'))
        flags |= O_EXCL;
    if (strchr(mode, 'e'))
        flags |= O_CLOEXEC;
    if (*mode != 'r')
        flags |= O_CREAT;
    if (*mode == 'w')
        flags |= O_TRUNC;
    if (*mode == 'a')
        flags |= O_APPEND;
    return flags;
}

FILE *fopen(const char *filename, const char *mode)
{
    FILE *f;
    int flags;

    if (!strchr("rwa", *mode)) {
        errno = EINVAL;
        return 0;
    }

    f = (FILE *)malloc(sizeof(FILE));

    flags = __fmodeflags(mode);
    // TODO: currently mode is unused in ax_open
    int fd = open(filename, flags, 0666);
    if (fd < 0)
        return NULL;
    f->fd = fd;

    return f;
}

char *fgets(char *restrict s, int n, FILE *restrict f)
{
    if (n == 0)
        return NULL;
    if (n == 1) {
        *s = '\0';
        return s;
    }

    int cnt = 0;
    while (cnt < n - 1) {
        char c;
        if (read(f->fd, (void *)&c, 1) > 0) {
            if (c != '\n')
                s[cnt++] = c;
            else{
                s[cnt++] = c;
                break;
            }
                
        } else
            break;
    }
    if(cnt==0){
        return NULL;
    }
    s[cnt] = '\0';
    return s;
}

size_t fread(void *restrict destv, size_t size, size_t nmemb, FILE *restrict f)
{
    size_t total = size * nmemb;
    size_t read_len = 0;
    size_t len = 0;
    do {
        len = read(f->fd, destv + read_len, total - read_len);
        if (len < 0)
            break;
        read_len += len;
    } while (len > 0);
    return read_len == size * nmemb ? nmemb : read_len / size;
}

size_t fwrite(const void *restrict src, size_t size, size_t nmemb, FILE *restrict f)
{
    size_t total = size * nmemb;
    size_t write_len = 0;
    size_t len = 0;
    do {
        len = write(f->fd, src + write_len, total - write_len);
        if (len < 0)
            break;
        write_len += len;
    } while (len > 0);
    return write_len == size * nmemb ? nmemb : write_len / size;
}

int fputs(const char *restrict s, FILE *restrict f)
{
    size_t l = strlen(s);
    return (fwrite(s, 1, l, f) == l) - 1;
}

int fclose(FILE *f)
{
    return close(f->fd);
}

int fileno(FILE *f)
{
    return f->fd;
}

int feof(FILE *f)
{
    unimplemented();
    return 0;
}

// TODO
int fseek(FILE *__stream, long __off, int __whence)
{
    unimplemented();
    return 0;
}

// TODO
off_t ftello(FILE *__stream)
{
    unimplemented();
    return 0;
}

// TODO
char *tmpnam(char *buf)
{
    unimplemented();
    return 0;
}

// TODO
void clearerr(FILE *f)
{
    unimplemented();
}

// TODO
int ferror(FILE *f)
{
    unimplemented();
    return 0;
}


FILE *freopen(const char *restrict filename, const char *restrict mode, FILE *restrict f)
{
    int fl = __fmodeflags(mode);
	FILE *f2;

	fflush(f);
	
	if (!filename) {
		if (fl&O_CLOEXEC)
            fcntl(f->fd, F_SETFD, FD_CLOEXEC);
		fl &= ~(O_CREAT|O_EXCL|O_CLOEXEC);
        if(fcntl(f->fd, F_SETFL, fl) < 0)
            goto fail;
	} else {
		f2 = fopen(filename, mode);
		if (!f2) goto fail;
		if (f2->fd == f->fd) f2->fd = -1; /* avoid closing in fclose */
		else if (dup3(f2->fd, f->fd, fl&O_CLOEXEC)<0) goto fail2;
		fclose(f2);
	}
	return f;

fail2:
	fclose(f2);
fail:
	fclose(f);
	return NULL;
}

// TODO
int fscanf(FILE *restrict f, const char *restrict fmt, ...)
{
    unimplemented();
    return 0;
}

// TODO
long ftell(FILE *f)
{
    unimplemented();
    return 0;
}

int getc(FILE *f)
{
    unimplemented();
    return 0;
}

int remove(const char *path)
{
    if(unlink(path) < 0) {
        return rmdir(path);
    }
    return 0;
}

// TODO
int setvbuf(FILE *restrict f, char *restrict buf, int type, size_t size)
{
    unimplemented();
    return 0;
}

// TODO
FILE *tmpfile(void)
{
    unimplemented();
    return NULL;
}

int ungetc(int c, FILE *f)
{
    unimplemented();
    return 0;
}

ssize_t getdelim(char **restrict s, size_t *restrict n, int delim, FILE *restrict f)
{
    unimplemented();
    return 0;
}

ssize_t getline(char **restrict s, size_t *restrict n, FILE *restrict f)
{
    return getdelim(s, n, '\n', f);
}

int __uflow(FILE *f)
{
    unimplemented();
    return 0;
}

int getc_unlocked(FILE *f)
{
    unimplemented();
    return 0;
}

FILE *fdopen(int fd, const char *mode)
{
    FILE *f;
    if (!strchr("rwa", *mode)) {
        errno = EINVAL;
        return 0;
    }

    if (!(f=malloc(sizeof *f))) return 0;
    f->buffer_len = 0;

    /* Apply close-on-exec flag */
	if (strchr(mode, 'e')) fcntl(fd, F_SETFD, FD_CLOEXEC);

    /* Set append mode on fd if opened for append */
    if (*mode == 'a') {
        int flags = fcntl(fd, F_GETFL);
        if (!(flags & O_APPEND))
            fcntl(fd, F_SETFL, flags | O_APPEND);
    }
    f->fd = fd;
    return f;
}

#endif // RUX_CONFIG_FS
