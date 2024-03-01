ARCH ?= x86_64
C_COMPILER := $(shell which $(CC))
CROSS_COMPILE_PATH := $(shell dirname $(C_COMPILER))/..
CXX_STD := c++20

main-obj := main.o
app-objs := cpp.o

$(APP)/$(app-objs): build_cpp
build_cpp: $(APP)/axbuild.mk $(APP)/main.cpp
	$(C_COMPILER) -o $(APP)/$(main-obj) -nostdlib -static -no-pie -c -std=$(CXX_STD) \
		$(APP)/main.cpp -I$(CROSS_COMPILE_PATH)/*-linux-musl/include/c++/*
	$(LD) -o $(app-objs) $(APP)/$(main-obj) -nostdlib -static -no-pie  -r -e main \
		$(CROSS_COMPILE_PATH)/*-linux-musl/lib/libstdc++.a \
		$(CROSS_COMPILE_PATH)/lib/gcc/*-linux-musl/*/libgcc_eh.a

clean_c::
	rm -rf $(app-objs)
	rm -rf $(APP)/$(main-obj)

.PHONY: build_cpp clean_c
