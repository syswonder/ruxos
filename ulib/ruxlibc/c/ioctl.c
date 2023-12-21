/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#include <stdarg.h>
#include <stdio.h>
#include <sys/ioctl.h>

int rux_ioctl(int fd, int cmd, size_t arg);

// TODO
int ioctl(int fd, int request, ...)
{
    unsigned long arg;
    va_list ap;
    va_start(ap, request);
    arg = va_arg(ap, unsigned long);
    va_end(ap);

    return rux_ioctl(fd, request, arg);
}
