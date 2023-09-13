#include<stdio.h>
#include<fcntl.h>
 
int main(){
    printf("test begin\n");
    char s[30];
    int fd=open("a.txt", O_RDWR | O_CREAT);
    if (fd==-1) {
        printf("can not create the file\n");
        return -1;
    }

    FILE *fp = fdopen(fd, "w");
    fprintf(fp,"mingrisoftminribook\n");
    fclose(fp);

    fd=open("a.txt", O_RDWR);
    fp = fdopen(fd, "r");
    fgets(s, 30, fp);
    printf("content is %s\n", s);
    fclose(fp);

    // fd=open("a.txt", O_RDWR | O_CREAT);
    // fp = fdopen(fd, "a+"); // 目前fcntl不支持
    // fprintf(fp,"newnewnew");
    // fgets(s, 30, fp);
    // printf("%s\n", s);
    // fclose(fp);

    printf("OK\n");
    return 0;
}