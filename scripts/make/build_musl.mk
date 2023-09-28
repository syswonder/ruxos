rust_lib_name := ruxmusl
rust_lib := target/$(TARGET)/$(MODE)/lib$(rust_lib_name).a

musl_version := 1.2.3

muslibc_dir := ulib/ruxmusl
build_dir := $(muslibc_dir)/build_musl_$(ARCH)
musl_dir := $(muslibc_dir)/musl-$(musl_version)
inc_dir := $(muslibc_dir)/install/include
c_lib := $(muslibc_dir)/install/lib/libc.a
libgcc :=

CFLAGS += -nostdinc -fno-builtin -ffreestanding -Wall
CFLAGS += -I$(CURDIR)/$(inc_dir)
LDFLAGS += -nostdlib -static -no-pie --gc-sections -T$(LD_SCRIPT)

ifeq ($(MODE), release)
  CFLAGS += -O3
endif

ifeq ($(ARCH), x86_64)
  LDFLAGS += --no-relax
else ifeq ($(ARCH), riscv64)
  CFLAGS += -march=rv64gc -mabi=lp64d -mcmodel=medany
endif

ifeq ($(findstring fp_simd,$(FEATURES)),)
  ifeq ($(ARCH), x86_64)
    CFLAGS += -mno-sse
  else ifeq ($(ARCH), aarch64)
    CFLAGS += -mgeneral-regs-only
  endif
else
  ifeq ($(ARCH), riscv64)
    # for compiler-rt fallbacks like `__addtf3`, `__multf3`, ...
    libgcc := $(shell $(CC) -print-libgcc-file-name)
  endif
endif

build_musl: 
ifeq ($(wildcard $(build_dir)),)
  ifeq ($(wildcard $(musl_dir)),)
	@echo "Download musl-1.2.3 source code"
	wget https://musl.libc.org/releases/musl-1.2.3.tar.gz -P $(muslibc_dir)
	tar -zxvf $(muslibc_dir)/musl-1.2.3.tar.gz -C $(muslibc_dir) && rm -f $(muslibc_dir)/musl-1.2.3.tar.gz
  endif
	mkdir -p $(build_dir)
	cd $(build_dir) && ../musl-1.2.3/configure --prefix=../install --exec-prefix=../ --syslibdir=../install/lib --disable-shared ARCH=$(RUX_ARCH) CC=$(CC) CROSS_COMPILE=$(CROSS_COMPILE) CFLAGS=$(CFLAGS)
	cd $(build_dir) && $(MAKE) -j && $(MAKE) install
endif

$(c_lib): build_musl

app-objs := main.o

-include $(APP)/axbuild.mk  # override `app-objs`

app-objs := $(addprefix $(APP)/,$(app-objs))

$(APP)/%.o: $(APP)/%.c build_musl
	$(call run_cmd,$(CC),$(CFLAGS) $(APP_CFLAGS) -c -o $@ $<)

$(rust_lib): _cargo_build

$(OUT_ELF): $(c_lib) $(rust_lib) $(libgcc) $(app-objs)
	@printf "    $(CYAN_C)Linking$(END_C) $(OUT_ELF)\n"
	$(call run_cmd,$(LD),$(LDFLAGS) $(c_lib) $(rust_lib) $(libgcc) $(app-objs) -o $@)

$(APP)/axbuild.mk: ;

.PHONY: build_musl
