#include <stdio.h>
#include <unistd.h>

int main(int argc, char** argv, char**envp) {
	execv(argv[0], argv);
	return 0;
}