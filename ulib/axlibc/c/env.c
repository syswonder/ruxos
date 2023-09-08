/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

char *environ_[2] = {"dummy", NULL};
char **environ = (char **)environ_;

char *getenv(const char *name)
{
    size_t l = strchrnul(name, '=') - name;
    if (l && !name[l] && environ)
        for (char **e = environ; *e; e++)
            if (!strncmp(name, *e, l) && l[*e] == '=')
                return *e + l + 1;
    return 0;
}

// TODO
int setenv(const char *__name, const char *__value, int __replace)
{
    unimplemented();
    return 0;
}

// TODO
int unsetenv(const char *__name)
{
    unimplemented();
    return 0;
}
