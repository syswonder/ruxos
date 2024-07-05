#include <iostream>

using namespace std;

// This is a dummy function to avoid linker errors as this example should be a static executable.
void* __dso_handle = NULL;

int main(int argc, char* argv[]) {
	cout << "Hello, wolrd!" << endl;
	return 0;
}

