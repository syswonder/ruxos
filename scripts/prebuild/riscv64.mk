# non-musl riscv64 still needs a usable bios
# instead of the non-funtioning default one
define run_prebuild
  git submodule update --init --recursive --remote patches/opensbi
endef

RISCV_BIOS := $(CURDIR)/patches/opensbi/build/platform/generic/firmware/fw_dynamic.bin

$(RISCV_BIOS): prebuild
	CROSS_COMPILE=riscv64-linux-musl- $(MAKE) -C patches/opensbi PLATFORM=generic

build: $(RISCV_BIOS)
