#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char** argv) {
	puts("Running argv tests...");
	if (argc != 3) {
		puts("args num is wrong");
		return -1;
	}
	if (strcmp(argv[0], "envtest") || strcmp(argv[1], "test1") || strcmp(argv[2], "test2")) {
		puts("argv is wrong");
		return -1;
	}
	if(argv[3] != NULL) {
		puts("argv is wrong");
		return -1;
	}
	puts("Argv tests run OK!");

	puts("Running environ tests...");
	char *env1 = "env1", *ex1 = "ex1", *ex2 = "ex_2";
    if(setenv(env1, ex1, 1) || strcmp(ex1, getenv(env1))) {
		puts("set new env is wrong");
		return -1;
	}
	if(setenv(env1, ex2, 1) || strcmp(ex2, getenv(env1))) {
		puts("set old env is wrong");
		return -1;
	}
	if(setenv(env1, ex1, 0) || strcmp(ex2, getenv(env1))) {
		puts("override the old env is wrong");
		return -1;
	}
	if(strcmp("hello", getenv("world")) || strcmp("world", getenv("hello"))) {
		puts("boot env is wrong");
		return -1;
	}
	puts("Environ tests run OK!");
    return 0;
}


