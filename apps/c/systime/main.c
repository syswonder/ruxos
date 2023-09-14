#include <stdio.h>
#include <sys/time.h>

int main()
{
    struct timeval tv;
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return 1;
    }

    printf("Seconds: %ld\n", tv.tv_sec);
    printf("Microseconds: %ld\n", tv.tv_usec);

    //usleep(3000000);
    sleep(3);

    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return 1;
    }

    printf("Seconds: %ld\n", tv.tv_sec);
    printf("Microseconds: %ld\n", tv.tv_usec);

    struct timeval new_time;
    new_time.tv_sec = 1731110400; 
    new_time.tv_usec = 0;

    // 使用 settimeofday 设置新的系统时间
    if (settimeofday(&new_time, NULL) != 0 ) {
        perror("settimeofday");
        return 1;
    }
    if (gettimeofday(&tv, NULL) != 0 ) {
        perror("gettimeofday");
        return 1;
    }

    printf("Seconds: %ld\n", tv.tv_sec);
    printf("Microseconds: %ld\n", tv.tv_usec);
    return 0;
}