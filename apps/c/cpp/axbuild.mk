CMAKE = cmake

ARCH ?= x86_64
C_COMPILER := $(shell which $(CC))
CXX_COMPILER := $(shell which $(CC))
AR := $(shell which $(AR))
RANLIB := $(shell which $(RANLIB))
CROSS_COMPILE_PATH := $(shell dirname $(C_COMPILER))/..
CXX_STD ?= 20

app-objs := std_benchmark.o
std_benchmark_dir := $(APP)/std-benchmark
std_benchmark_build = $(std_benchmark_dir)/build

bench ?= all
benches_available := $(wildcard $(std_benchmark_dir)/cxx/*.bench.cpp)
benches_available := $(patsubst $(std_benchmark_dir)/cxx/%.bench.cpp,%,$(benches_available))

$(std_benchmark_dir):
	@echo "Download std-benchmark source code"
	cd $(APP)/ && git clone --recursive https://github.com/hiraditya/std-benchmark
	patch -p1 -N -d $(std_benchmark_dir) --no-backup-if-mismatch -r - < $(APP)/std_benchmark.patch

$(APP)/$(app-objs): build_std-benchmark
build_std-benchmark: $(std_benchmark_dir) $(APP)/axbuild.mk
	cd $(std_benchmark_dir) && mkdir -p build && cd build && \
		$(CMAKE) .. -DCMAKE_CXX_STANDARD=$(CXX_STD) -DCMAKE_C_COMPILER=$(C_COMPILER) -DCMAKE_CXX_COMPILER=$(CXX_COMPILER) -DCMAKE_AR=$(AR) -DCMAKE_RANLIB=$(RANLIB) \
			-DENABLE_C_BENCHMARKS=OFF -DENABLE_C_VS_CXX_BENCHMARKS=OFF -DENABLE_COMPILER_VS_PROGRAMMER=OFF -DBENCHMARK_ENABLE_TESTING=OFF && \
		$(MAKE) -j
	mkdir -p $(std_benchmark_build)/libgcc && cd $(std_benchmark_build)/libgcc && \
		ln -s -f $(CROSS_COMPILE_PATH)/lib/gcc/*-linux-musl/*/libgcc.a ./ && \
		$(AR) x libgcc.a _clrsbsi2.o
ifeq ($(bench), all)
	$(error "Running all benches automatically is not supported, please add 'bench=' arg. \
		Available benches: $(benches_available)")
endif
ifneq ($(filter $(bench),$(benches_available)),)
	$(LD) -o $(app-objs) -nostdlib -static -no-pie -r -e main \
		$(std_benchmark_build)/cxx/lib$(bench).bench.cpp.out.a \
		$(std_benchmark_build)/benchmark/src/libbenchmark.a \
		$(CROSS_COMPILE_PATH)/*-linux-musl/lib/libstdc++.a \
		$(CROSS_COMPILE_PATH)/lib/gcc/*-linux-musl/*/libgcc_eh.a \
		$(std_benchmark_build)/libgcc/_clrsbsi2.o
else
	$(error "Available benches: $(benches_available)")
endif

clean_c::
	rm -rf $(std_benchmark_build)/

.PHONY: build_std-benchmark clean_c


