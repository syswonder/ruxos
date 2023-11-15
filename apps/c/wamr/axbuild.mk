wamr-version := 6dbfeb25dd164c0ffcec21806e1c1cd0dff27c58
wamr-dir := $(APP)/wasm-micro-runtime-$(wamr-version)

app-objs := wamr.o

LIBWAMR_SRC := $(wamr-dir)
LIBWAMR_BASE := $(APP)

# TODO: -mindirect-branch-register is only needed for x86_64
WAMR_C_FLAGS = -fno-builtin -ffreestanding -std=gnu99 -ffunction-sections -fdata-sections -Wall -Wno-unused-parameter -Wno-pedantic -fPIC -Wall -Wextra -Wformat -Wformat-security -Wshadow -O3 -DNDEBUG -fPIE
WAMR_ASM_FLAGS = -fno-builtin -ffreestanding -O3 -DNDEBUG -fPIC

WAMR_C_DEFINES = -DBUILD_TARGET_AARCH64  -DBH_FREE=wasm_runtime_free -DBH_MALLOC=wasm_runtime_malloc -DBH_PLATFORM_LINUX -DWASM_DISABLE_HW_BOUND_CHECK=1 -DWASM_DISABLE_STACK_HW_BOUND_CHECK=1 -DWASM_DISABLE_WAKEUP_BLOCKING_OP=1 -DWASM_DISABLE_WRITE_GS_BASE=1 -DWASM_ENABLE_AOT=1 -DWASM_ENABLE_BULK_MEMORY=1 -DWASM_ENABLE_FAST_INTERP=1 -DWASM_ENABLE_INTERP=1 -DWASM_ENABLE_LIBC_BUILTIN=1 -DWASM_ENABLE_LIBC_WASI=1 -DWASM_ENABLE_MINI_LOADER=0 -DWASM_ENABLE_MODULE_INST_CONTEXT=1 -DWASM_ENABLE_MULTI_MODULE=0 -DWASM_ENABLE_SHARED_MEMORY=0 -DWASM_ENABLE_SIMD=1
# WAMR_C_DEFINES = -DBUILD_TARGET_AARCH64 -DBH_FREE=wasm_runtime_free -DBH_MALLOC=wasm_runtime_malloc -DBH_PLATFORM_LINUX -DWASM_DISABLE_HW_BOUND_CHECK=0 -DWASM_DISABLE_STACK_HW_BOUND_CHECK=0 -DWASM_DISABLE_WAKEUP_BLOCKING_OP=0 -DWASM_DISABLE_WRITE_GS_BASE=1 -DWASM_ENABLE_AOT=1 -DWASM_ENABLE_BULK_MEMORY=1 -DWASM_ENABLE_FAST_INTERP=1 -DWASM_ENABLE_INTERP=1 -DWASM_ENABLE_LIBC_BUILTIN=1 -DWASM_ENABLE_LIBC_WASI=1 -DWASM_ENABLE_MINI_LOADER=0 -DWASM_ENABLE_MODULE_INST_CONTEXT=1 -DWASM_ENABLE_MULTI_MODULE=0 -DWASM_ENABLE_SHARED_MEMORY=0 -DWASM_ENABLE_SIMD=1

C_INCLUDES = -I${LIBWAMR_SRC}/core/iwasm/interpreter \
				-I${LIBWAMR_SRC}/core/iwasm/aot \
				-I${LIBWAMR_SRC}/core/iwasm/libraries/libc-builtin \
				-I${LIBWAMR_SRC}/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/include \
				-I${LIBWAMR_SRC}/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/src \
				-I${LIBWAMR_SRC}/product-mini/platforms/linux/../../../core/iwasm/include \
				-I${LIBWAMR_SRC}/core/shared/platform/linux \
				-I${LIBWAMR_SRC}/core/shared/platform/linux/../include \
				-I${LIBWAMR_SRC}/core/shared/platform/common/libc-util \
				-I${LIBWAMR_SRC}/core/shared/mem-alloc \
				-I${LIBWAMR_SRC}/core/iwasm/common \
				-I${LIBWAMR_SRC}/core/shared/utils \
				-I${LIBWAMR_SRC}/core/shared/utils/uncommon

platform_src := $(LIBWAMR_SRC)/core/shared/platform/linux/platform_init.c \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_blocking_op.c \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_clock.c       \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_file.c        \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_malloc.c      \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_memmap.c      \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_sleep.c       \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_socket.c      \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_thread.c      \
				$(LIBWAMR_SRC)/core/shared/platform/common/posix/posix_time.c        \
				$(LIBWAMR_SRC)/core/shared/platform/common/libc-util/libc_errno.c

mem_alloc_src := $(LIBWAMR_SRC)/core/shared/mem-alloc/ems/ems_alloc.c \
				 $(LIBWAMR_SRC)/core/shared/mem-alloc/ems/ems_hmu.c   \
				 $(LIBWAMR_SRC)/core/shared/mem-alloc/ems/ems_kfc.c   \
				 $(LIBWAMR_SRC)/core/shared/mem-alloc/mem_alloc.c

utils_src := $(LIBWAMR_SRC)/core/shared/utils/bh_assert.c		\
				$(LIBWAMR_SRC)/core/shared/utils/bh_bitmap.c         \
				$(LIBWAMR_SRC)/core/shared/utils/bh_common.c         \
				$(LIBWAMR_SRC)/core/shared/utils/bh_hashmap.c        \
				$(LIBWAMR_SRC)/core/shared/utils/bh_list.c           \
				$(LIBWAMR_SRC)/core/shared/utils/bh_log.c            \
				$(LIBWAMR_SRC)/core/shared/utils/bh_queue.c          \
				$(LIBWAMR_SRC)/core/shared/utils/bh_vector.c         \
				$(LIBWAMR_SRC)/core/shared/utils/runtime_timer.c

libc_builtin_src := $(LIBWAMR_SRC)/core/iwasm/libraries/libc-builtin/libc_builtin_wrapper.c

libc_wasi_src := $(LIBWAMR_SRC)/core/iwasm/libraries/libc-wasi/libc_wasi_wrapper.c \
					$(LIBWAMR_SRC)/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/src/blocking_op.c\
					$(LIBWAMR_SRC)/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/src/posix.c   \
					$(LIBWAMR_SRC)/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/src/random.c  \
					$(LIBWAMR_SRC)/core/iwasm/libraries/libc-wasi/sandboxed-system-primitives/src/str.c

iwasm_common_src := $(LIBWAMR_SRC)/core/iwasm/common/wasm_application.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_blocking_op.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_c_api.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_exec_env.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_memory.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_native.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_runtime_common.c \
					$(LIBWAMR_SRC)/core/iwasm/common/wasm_shared_memory.c
iwasm_common_src_s := $(LIBWAMR_SRC)/core/iwasm/common/arch/invokeNative_aarch64_simd.s

iwasm_interp_src := $(LIBWAMR_SRC)/core/iwasm/interpreter/wasm_interp_fast.c \
					$(LIBWAMR_SRC)/core/iwasm/interpreter/wasm_loader.c \
					$(LIBWAMR_SRC)/core/iwasm/interpreter/wasm_runtime.c

iwasm_aot_src := $(LIBWAMR_SRC)/core/iwasm/aot/aot_intrinsic.c \
					$(LIBWAMR_SRC)/core/iwasm/aot/aot_loader.c \
					$(LIBWAMR_SRC)/core/iwasm/aot/aot_runtime.c \
					$(LIBWAMR_SRC)/core/iwasm/aot/arch/aot_reloc_x86_64.c

LIBWAMR_SRCS := $(platform_src) $(mem_alloc_src) $(utils_src) $(libc_builtin_src) $(libc_wasi_src) $(iwasm_common_src) $(iwasm_interp_src) $(iwasm_aot_src)
LIBWAMR_SRCS_s := $(iwasm_common_src_s)

LIBWAMR_SRCS += $(LIBWAMR_SRC)/product-mini/platforms/linux/main.c
LIBWAMR_SRCS += $(LIBWAMR_SRC)/core/shared/utils/uncommon/bh_getopt.c
LIBWAMR_SRCS += $(LIBWAMR_SRC)/core/shared/utils/uncommon/bh_read_file.c

objs := $(LIBWAMR_SRCS:.c=.o)
objs_s := $(LIBWAMR_SRCS_s:.s=.o)

$(wamr-dir):
	@echo "Download wamr source code"
	wget https://github.com/intel/wasm-micro-runtime/archive/$(wamr-version).tar.gz -P $(APP)
	tar -zxf $(APP)/$(wamr-version).tar.gz -C $(APP) && rm -f $(APP)/$(wamr-version).tar.gz
	cd $(wamr-dir) && git init && git add .
	patch -p1 -N -d $(wamr-dir) --no-backup-if-mismatch -r - < $(APP)/wamr.patch

$(APP)/$(app-objs): build_wamr

$(objs_s): %.o: %.s
	$(call run_cmd, $(CC), $(WAMR_C_DEFINES) $(C_INCLUDES) $(WAMR_ASM_FLAGS) -c -o $@ $<)

$(objs): %.o: %.c $(LIBWAMR_SRC)/product-mini/platforms/posix/main.c
	$(call run_cmd, $(CC), $(WAMR_C_DEFINES) $(C_INCLUDES) $(WAMR_C_FLAGS) -c -o $@ $<)

build_wamr: $(wamr-dir) $(objs) $(objs_s)
# -r means relocatable
	$(call run_cmd, $(LD), $(LDFLAGS) -r -e main $(objs) $(objs_s) -o $(app-objs))

clean_c::
	rm -f $(objs) $(objs_s)

.PHONY: build_wamr clean_c
