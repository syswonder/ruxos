#include <stdio.h> 
#include<fcntl.h>
#include<unistd.h>
int main() 
{ 
    printf("test begin\n");
    int fd=open("in.txt", O_RDWR | O_CREAT);
    int fd2 = open("out.txt", O_RDWR | O_CREAT);
    // close(fd2);
    FILE *fp = fdopen(fd, "w");
    int ret = fprintf(fp,"1 2 3 4\n");
    fclose(fp);
    if(!freopen("out.txt", "w", fp)) printf("fail 1\n");
    fprintf(fp, "1 2 3 4\n");
    fclose(fp);

    char s[30];
    fd=open("in.txt", O_RDWR);
    fp = fdopen(fd, "w+");
    fgets(s, 30, fp);
    printf("in.txt is %s\n", s);
    fclose(fp);

    fd=open("out.txt", O_RDWR | O_CREAT);
    fp = fdopen(fd, "w+");
    fgets(s, 30, fp);
    printf("out.txt is %s\n", s);
    fclose(fp);
    return 0; 
} 