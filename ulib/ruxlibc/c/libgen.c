/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <libgen.h>
#include <string.h>

char *dirname(char *s)
{
    size_t i;
    if (!s || !*s)
        return ".";
    i = strlen(s) - 1;
    for (; s[i] == '/'; i--)
        if (!i)
            return "/";
    for (; s[i] != '/'; i--)
        if (!i)
            return ".";
    for (; s[i] == '/'; i--)
        if (!i)
            return "/";
    s[i + 1] = 0;
    return s;
}

char *basename(char *s)
{
    size_t i;
    if (!s || !*s)
        return ".";
    i = strlen(s) - 1;
    for (; i && s[i] == '/'; i--) s[i] = 0;
    for (; i && s[i - 1] != '/'; i--)
        ;
    return s + i;
}
