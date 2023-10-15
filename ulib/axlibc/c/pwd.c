/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <pwd.h>
#include <stdio.h>
#include <string.h>
#include <errno.h>
#include <sys/types.h>

/* Default passwd */
static struct passwd pw__ = {
	.pw_name = AX_DEFAULT_USER,
	.pw_passwd = AX_DEFAULT_PASS,
	.pw_uid = AX_DEFAULT_UID,
	.pw_gid = AX_DEFAULT_GID,
	.pw_gecos = AX_DEFAULT_USER,
	.pw_dir = "/",
	.pw_shell = "",
};

// TODO
int getpwnam_r(const char *name, struct passwd *pw, char *buf, size_t size, struct passwd **res)
{
    unimplemented();
    return 0;
}

// TODO
int getpwuid_r(uid_t uid, struct passwd *pw, char *buf, size_t size, struct passwd **res)
{
    unimplemented();
    return 0;
}

struct passwd *getpwnam(const char *name)
{
	struct passwd *pwd;

	if (name && !strcmp(name, pw__.pw_name))
		pwd = &pw__;
	else {
		pwd = NULL;
		errno = ENOENT;
	}

	return pwd;
}
