#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
pthread_mutex_t mutex;
pthread_cond_t condvar;
int A = 0;
void *first(void *arg)
{
    sleep(5);
    puts("First work, Change A --> 1 and wakeup Second");
    pthread_mutex_lock(&mutex);
    A = 1;
    pthread_cond_signal(&condvar);
    pthread_mutex_unlock(&mutex);
    return NULL;

}
void *second(void *arg)
{
    puts("Second want to continue,but need to wait A=1");
    pthread_mutex_lock(&mutex);
    while (A == 0) {
        printf("Second: A is {}", A);
        pthread_cond_wait(&condvar, &mutex);
    }
    printf("A is {}, Second can work now", A);
    pthread_mutex_unlock(&mutex);
    return NULL;
}

int main()
{
    pthread_t t1, t2;
    pthread_mutex_init(&mutex, NULL);
    pthread_cond_init(&condvar, NULL);

    pthread_create(&t1, NULL, first, NULL);
    pthread_create(&t2, NULL, second, NULL);

    pthread_join(t1, NULL);
    pthread_join(t2, NULL);
    puts("(C)Pthread Condvar test finish!");
    return 0;
}