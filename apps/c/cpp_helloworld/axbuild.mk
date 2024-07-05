ARCH ?= x86_64
C_COMPILER := $(shell which $(CC))
CXX_COMPILER := $(shell which $(CC))
AR := $(shell which $(AR))
RANLIB := $(shell which $(RANLIB))
CROSS_COMPILE_PATH := $(shell dirname $(C_COMPILER))/..

app-objs := main.o

$(APP)/$(app-objs): $(APP)/axbuild.mk
	$(CXX_COMPILER) -c -o $(app-objs) $(APP)/main.cpp
	$(LD) -o $(app-objs) -nostdlib -static -no-pie -r -e main \
		$(app-objs) \
		$(CROSS_COMPILE_PATH)/*-linux-musl/lib/libstdc++.a \
		$(CROSS_COMPILE_PATH)/lib/gcc/*-linux-musl/*/libgcc_eh.a

clean_c::
	rm -rf $(APP)/$(app-objs) 

.PHONY: $(APP)/$(app-objs) clean_c
