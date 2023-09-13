#include<stdio.h> 
#include<fcntl.h>
#include<unistd.h>
#include<errno.h>
int main()
{
    int ret;
    ret = rmdir("lgw");
    printf("ret is %d\n", ret);
    ret = mkdir("lgw", 0);
    printf("ret is %d\n", ret);
    int fd = open("lgw/a.txt", O_RDWR | O_CREAT);
    close(fd);
    // ret = rmdir("lgw");
    // printf("ret is %d\n", ret);
    // ret = mkdir("lgw",0);
    // printf("ret is %d\n", ret);
    ret = remove("lgw/a.txt");
    printf("ret is %d, err is %d\n", ret, errno);
    ret = remove("lgw");
    printf("ret is %d, err is %d\n", ret, errno);
    return 0;
}