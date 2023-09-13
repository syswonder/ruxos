#include <netdb.h>
#include <stdio.h>
#include <string.h>
#include <arpa/inet.h>
#include <netinet/in.h>
#include <sys/socket.h>
#include <sys/uio.h>
const char request1[] = "\
GET / HTTP/1.1\r\n";

const char request2[] = "Host: ident.me\r\n\
Accept: */*\r\n\
\r\n";

int main()
{
    puts("Hello, ArceOS C HTTP client!");
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
    // 将数值格式转化为点分十进制表示
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
    struct iovec iovs[2] = {        
                                { .iov_base = request1, .iov_len = strlen(request1)},
                                { .iov_base = request2, .iov_len = strlen(request2)}
                            };
    struct msghdr mg = {
        .msg_iov = iovs,
        .msg_iovlen = 2
    };
    int num = sendmsg(sock, &mg, 0);
    printf("num is %d\n", num); 
    if (num == -1) {
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
    printf("test success\n");
    return 0;
}
