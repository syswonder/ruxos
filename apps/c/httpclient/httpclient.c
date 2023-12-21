/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#include <arpa/inet.h>
#include <netdb.h>
#include <netinet/in.h>
#include <stdio.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/uio.h>

const char request[] = "\
GET / HTTP/1.1\r\n\
Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

char request1[] = "\
GET / HTTP/1.1\r\n";

char request2[] = "Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

int main()
{
    puts("Hello, Ruxos C HTTP client!");
    int sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (sock == -1) {
        perror("socket() error");
        return -1;
    }
    struct addrinfo *res;

    if (getaddrinfo("ident.me", NULL, NULL, &res) != 0) {
        perror("getaddrinfo() error");
        return -1;
    }
    char str[INET_ADDRSTRLEN];
    if (inet_ntop(AF_INET, &(((struct sockaddr_in *)(res->ai_addr))->sin_addr), str,
                  INET_ADDRSTRLEN) == NULL) {
        perror("inet_ntop() error");
        return -1;
    }
    printf("IP: %s\n", str);
    ((struct sockaddr_in *)(res->ai_addr))->sin_port = htons(80);
    if (connect(sock, res->ai_addr, sizeof(*(res->ai_addr))) != 0) {
        perror("connect() error");
        return -1;
    }
    char rebuf[2000] = {};
    if (send(sock, request, strlen(request), 0) == -1) {
        perror("send() error");
        return -1;
    }
    ssize_t l = recv(sock, rebuf, 2000, 0);
    if (l == -1) {
        perror("recv() error");
        return -1;
    }
    rebuf[l] = '\0';
    printf("%s\n", rebuf);
	// test sendmsg
	struct iovec iovs[2] = {        
							{ .iov_base = request1, .iov_len = strlen(request1)},
							{ .iov_base = request2, .iov_len = strlen(request2)}
						};
    struct msghdr mg = {
        .msg_iov = iovs,
        .msg_iovlen = 2
    };
    int num = sendmsg(sock, &mg, 0);
	if (num == -1) {
		perror("sendmsg() error");
        return -1;
	}
    freeaddrinfo(res);

    return 0;
}
