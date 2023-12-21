/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include "nscd.h"
#include <byteswap.h>
#include <errno.h>
#include <limits.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/uio.h>
#include <unistd.h>

static const struct {
    short sun_family;
    char sun_path[21];
} addr = {AF_UNIX, "/var/run/nscd/socket"};

FILE *__nscd_query(int32_t req, const char *key, int32_t *buf, size_t len, int *swap)
{
    size_t i;
    int fd;
    FILE *f = 0;
    int32_t req_buf[REQ_LEN] = {NSCDVERSION, req, strnlen(key, LOGIN_NAME_MAX) + 1};
    struct msghdr msg = {
        .msg_iov = (struct iovec[]){{&req_buf, sizeof(req_buf)}, {(char *)key, strlen(key) + 1}},
        .msg_iovlen = 2};
    int errno_save = errno;

    *swap = 0;
retry:
    memset(buf, 0, len);
    buf[0] = NSCDVERSION;

    fd = socket(PF_UNIX, SOCK_STREAM | SOCK_CLOEXEC, 0);
    if (fd < 0) {
        if (errno == EAFNOSUPPORT) {
            f = fopen("/dev/null", "re");
            if (f)
                errno = errno_save;
            return f;
        }
        return 0;
    }

    if (!(f = fdopen(fd, "r"))) {
        close(fd);
        return 0;
    }

    if (req_buf[2] > LOGIN_NAME_MAX)
        return f;

    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0) {
        if (errno == EACCES || errno == ECONNREFUSED || errno == ENOENT) {
            errno = errno_save;
            return f;
        }
        goto error;
    }

    if (sendmsg(fd, &msg, MSG_NOSIGNAL) < 0)
        goto error;

    if (!fread(buf, len, 1, f)) {
        if (ferror(f))
            goto error;
        if (!*swap) {
            fclose(f);
            for (i = 0; i < sizeof(req_buf) / sizeof(req_buf[0]); i++) {
                req_buf[i] = bswap_32(req_buf[i]);
            }
            *swap = 1;
            goto retry;
        } else {
            errno = EIO;
            goto error;
        }
    }

    if (*swap) {
        for (i = 0; i < len / sizeof(buf[0]); i++) {
            buf[i] = bswap_32(buf[i]);
        }
    }

    if (buf[0] != NSCDVERSION) {
        errno = EIO;
        goto error;
    }

    return f;
error:
    fclose(f);
    return 0;
}
