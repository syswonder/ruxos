/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <errno.h>
#include <fcntl.h>
#include <stdio.h>
#include <string.h>
#include <sys/stat.h>
#include <unistd.h>

int main()
{
    int ret;
    ret = rmdir("filetest");
    ret = mkdir("filetest", 0);
    int fd = open("filetest/a.txt", O_RDWR | O_CREAT);
    if (fd == -1) {
        perror("can not create the file\n");
        return -1;
    }

    puts("rmdir, mkdir, open success!");

    FILE *fp = fdopen(fd, "w");
    fprintf(fp, "1 2 3 4\n");
    fprintf(fp, "5 6 7 8\n");
    fclose(fp);

    char s[50];
    fd = open("filetest/a.txt", O_RDWR);
    fp = fdopen(fd, "r");

    fgets(s, 50, fp);
    if (strcmp("1 2 3 4\n", s)) {
        perror("fdopen and freopen failed");
        return -1;
    }
    puts("first fgets success!");

    fgets(s, 50, fp);
    if (strcmp("5 6 7 8\n", s)) {
        perror("fdopen and freopen failed");
        return -1;
    }
    puts("second fgets success!");

    fclose(fp);
    ret = remove("filetest/a.txt");
    if (ret == -1) {
        perror("remove file error");
        return -1;
    }
    ret = remove("filetest");
    if (ret == -1) {
        perror("remove dir error");
        return -1;
    }
    puts("remove file and dir success!");

    puts("filetest success!");
    return 0;
}
