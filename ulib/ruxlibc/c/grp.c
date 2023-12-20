/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <stdio.h>
#include <grp.h>
#include <pwd.h>
#include <string.h>
#include <errno.h>

/* Group members */
static char *g_members__[] = { RUX_DEFAULT_USER, NULL };

/* Default group */
static struct group g__ = {
	.gr_name = RUX_DEFAULT_GROUP,
	.gr_passwd = RUX_DEFAULT_PASS,
	.gr_gid = RUX_DEFAULT_GID,
	.gr_mem = g_members__,
};

// TODO
int initgroups(const char *user, gid_t group)
{
    unimplemented();
    return 0;
}

struct group *getgrnam(const char *name)
{
	struct group *res;

	if (name && !strcmp(name, g__.gr_name))
		res = &g__;
	else {
		res = NULL;
		errno = ENOENT;
	}

	return res;
}
