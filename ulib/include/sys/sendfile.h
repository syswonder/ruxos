/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS
 * OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A
 * PARTICULAR PURPOSE. See the Mulan PSL v2 for more details.
 */

#ifndef _SYS_SENDFILE_H
#define _SYS_SENDFILE_H

#include <features.h>
#include <unistd.h>

ssize_t sendfile(int, int, off_t *, size_t);

#if defined(_LARGEFILE64_SOURCE) || defined(_GNU_SOURCE)
#define sendfile64 sendfile
#define off64_t    off_t
#endif

#endif
