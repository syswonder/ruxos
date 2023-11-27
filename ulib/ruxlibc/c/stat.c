/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stdio.h>
#include <sys/stat.h>
#include <sys/types.h>

// TODO:
int fchmod(int fd, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO
int chmod(const char *path, mode_t mode)
{
    unimplemented();
    return 0;
}

// TODO
mode_t umask(mode_t mask)
{
    unimplemented("mask: %d", mask);
    return 0;
}

// TODO
int fstatat(int fd, const char *restrict path, struct stat *restrict st, int flag)
{
    unimplemented();
    return 0;
}
