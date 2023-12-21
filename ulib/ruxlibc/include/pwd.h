/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#ifndef _PWD_H
#define _PWD_H

#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>

#define NSCDVERSION 2
#define GETPWBYNAME 0
#define GETPWBYUID  1
#define GETGRBYNAME 2
#define GETGRBYGID  3
#define GETINITGR   15

#define PWVERSION   0
#define PWFOUND     1
#define PWNAMELEN   2
#define PWPASSWDLEN 3
#define PWUID       4
#define PWGID       5
#define PWGECOSLEN  6
#define PWDIRLEN    7
#define PWSHELLLEN  8
#define PW_LEN      9

#define REQVERSION 0
#define REQTYPE    1
#define REQKEYLEN  2
#define REQ_LEN    3

#define RUX_DEFAULT_USER "root"
#define RUX_DEFAULT_GROUP "root"
#define RUX_DEFAULT_PASS ""
#define RUX_DEFAULT_UID 0
#define RUX_DEFAULT_GID 0

struct passwd {
    char *pw_name;
    char *pw_passwd;
    uid_t pw_uid;
    gid_t pw_gid;
    char *pw_gecos;
    char *pw_dir;
    char *pw_shell;
};

int getpwuid_r(uid_t, struct passwd *, char *, size_t, struct passwd **);
int getpwnam_r(const char *, struct passwd *, char *, size_t, struct passwd **);

struct passwd *getpwnam (const char *);

#endif // _PWD_H
