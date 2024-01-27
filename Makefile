# Available arguments:
# * General options:
#     - `ARCH`: Target architecture: x86_64, riscv64, aarch64
#     - `PLATFORM`: Target platform in the `platforms` directory
#     - `SMP`: Number of CPUs
#     - `MODE`: Build mode: release, debug, reldebug
#     - `LOG:` Logging level: warn, error, info, debug, trace
#     - `V`: Verbose level: (empty), 1, 2
#	    - `ARGS`: Command-line arguments separated by comma. Only available when feature `alloc` is enabled.
#	    - `ENVS`: Environment variables, separated by comma between key value pairs. Only available when feature `alloc` is enabled.
# * App options:
#     - `A` or `APP`: Path to the application
#     - `FEATURES`: Features of Ruxos modules to be enabled.
#     - `APP_FEATURES`: Features of (rust) apps to be enabled.
# * QEMU options:
#     - `BLK`: Enable storage devices (virtio-blk)
#     - `NET`: Enable network devices (virtio-net)
#     - `GRAPHIC`: Enable display devices and graphic output (virtio-gpu)
#     - `V9P`: Enable virtio-9p devices
#     - `BUS`: Device bus type: mmio, pci
#     - `DISK_IMG`: Path to the virtual disk image
#     - `ACCEL`: Enable hardware acceleration (KVM on linux)
#     - `QEMU_LOG`: Enable QEMU logging (log file is "qemu.log")
#     - `NET_DUMP`: Enable network packet dump (log file is "netdump.pcap")
#     - `NET_DEV`: QEMU netdev backend types: user, tap
#     - `START_PORT`: The starting port number for the open ports in QEMU (default is port 5555)
#     - `PORT_NUM`: The number of open ports in QEMU (default is 5)
# * 9P options:
#     - `V9P_PATH`: Host path for backend of virtio-9p
#     - `NET_9P_ADDR`: Server address and port for 9P netdev 
#     - `ANAME_9P`: Path for root of 9pfs(parameter of TATTACH for root)
#     - `PROTOCOL_9P`: Default protocol version selected for 9P
# * Network options:
#     - `IP`: Ruxos IPv4 address (default is 10.0.2.15 for QEMU user netdev)
#     - `GW`: Gateway IPv4 address (default is 10.0.2.2 for QEMU user netdev)
# * Libc options:
#     - `MUSL`: Link C app with musl libc

# General options
ARCH ?= x86_64
PLATFORM ?=
SMP ?= 1
MODE ?= release
LOG ?= warn
V ?=

# App options
A ?= apps/c/helloworld
APP ?= $(A)
FEATURES ?=
APP_FEATURES ?=

# QEMU options
BLK ?= n
NET ?= n
GRAPHIC ?= n
V9P ?= n
BUS ?= mmio
RISCV_BIOS ?= $(shell realpath ./platforms/riscv/fw_dynamic.bin)

DISK_IMG ?= disk.img
QEMU_LOG ?= n
NET_DUMP ?= n
NET_DEV ?= user
V9P_PATH ?= ./
NET_9P_ADDR ?= 127.0.0.1:564
ANAME_9P ?= ./
PROTOCOL_9P ?= 9P2000.L

START_PORT ?= 5555
PORTS_NUM ?= 5
# Network options
IP ?= 10.0.2.15
GW ?= 10.0.2.2

# args and envs
ARGS ?= 
ENVS ?= 

# Libc options
MUSL ?= n

# App type
ifeq ($(wildcard $(APP)),)
  $(error Application path "$(APP)" is not valid)
endif

ifneq ($(wildcard $(APP)/Cargo.toml),)
  APP_TYPE := rust
else
  APP_TYPE := c
endif

# Architecture, platform and target
ifneq ($(filter $(MAKECMDGOALS),unittest unittest_no_fail_fast),)
  PLATFORM_NAME :=
else ifneq ($(PLATFORM),)
  # `PLATFORM` is specified, override the `ARCH` variables
  builtin_platforms := $(patsubst platforms/%.toml,%,$(wildcard platforms/*))
  ifneq ($(filter $(PLATFORM),$(builtin_platforms)),)
    # builtin platform
    PLATFORM_NAME := $(PLATFORM)
    _arch := $(word 1,$(subst -, ,$(PLATFORM)))
  else ifneq ($(wildcard $(PLATFORM)),)
    # custom platform, read the "platform" field from the toml file
    PLATFORM_NAME := $(shell cat $(PLATFORM) | sed -n 's/^platform = "\([a-z0-9A-Z_\-]*\)"/\1/p')
    _arch := $(shell cat $(PLATFORM) | sed -n 's/^arch = "\([a-z0-9A-Z_\-]*\)"/\1/p')
  else
    $(error "PLATFORM" must be one of "$(builtin_platforms)" or a valid path to a toml file)
  endif
  ifeq ($(origin ARCH),command line)
    ifneq ($(ARCH),$(_arch))
      $(error "ARCH=$(ARCH)" is not compatible with "PLATFORM=$(PLATFORM)")
    endif
  endif
  ARCH := $(_arch)
endif

ifeq ($(ARCH), x86_64)
  # Don't enable kvm for WSL/WSL2.
  ACCEL ?= $(if $(findstring -microsoft, $(shell uname -r | tr '[:upper:]' '[:lower:]')),n,y)
  PLATFORM_NAME ?= x86_64-qemu-q35
  TARGET := x86_64-unknown-none
  BUS := pci
else ifeq ($(ARCH), riscv64)
  ACCEL ?= n
  PLATFORM_NAME ?= riscv64-qemu-virt
  TARGET := riscv64gc-unknown-none-elf
else ifeq ($(ARCH), aarch64)
  ACCEL ?= n
  PLATFORM_NAME ?= aarch64-qemu-virt
  TARGET := aarch64-unknown-none-softfloat
else
  $(error "ARCH" must be one of "x86_64", "riscv64", or "aarch64")
endif

export RUX_ARCH=$(ARCH)
export RUX_PLATFORM=$(PLATFORM_NAME)
export RUX_SMP=$(SMP)
export RUX_MODE=$(MODE)
export RUX_LOG=$(LOG)
export RUX_TARGET=$(TARGET)
export RUX_IP=$(IP)
export RUX_GW=$(GW)
export RUX_9P_ADDR = $(NET_9P_ADDR)
export RUX_ANAME_9P = $(ANAME_9P)
export RUX_PROTOCOL_9P = $(PROTOCOL_9P)
export RUX_MUSL=$(MUSL)

# Binutils
CROSS_COMPILE ?= $(ARCH)-linux-musl-
CC := $(CROSS_COMPILE)gcc
AR := $(CROSS_COMPILE)ar
RANLIB := $(CROSS_COMPILE)ranlib
LD := rust-lld -flavor gnu

OBJDUMP ?= rust-objdump -d --print-imm-hex --x86-asm-syntax=intel
OBJCOPY ?= rust-objcopy --binary-architecture=$(ARCH)
GDB ?= gdb-multiarch

# Paths
OUT_DIR ?= $(APP)

APP_NAME := $(shell basename $(APP))
LD_SCRIPT := $(CURDIR)/modules/ruxhal/linker_$(PLATFORM_NAME).lds
OUT_ELF := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM_NAME).elf
OUT_BIN := $(OUT_DIR)/$(APP_NAME)_$(PLATFORM_NAME).bin

all: build

include scripts/make/utils.mk
include scripts/make/build.mk
include scripts/make/qemu.mk
include scripts/make/test.mk
ifeq ($(PLATFORM_NAME), aarch64-raspi4)
  include scripts/make/raspi4.mk
else ifeq ($(PLATFORM_NAME), aarch64-bsta1000b)
  include scripts/make/bsta1000b-fada.mk
endif

build: $(OUT_DIR) $(OUT_BIN)

disasm:
	$(OBJDUMP) $(OUT_ELF) | less

run: build
	$(call run_qemu)

justrun:
	$(call run_qemu)

debug: build
	$(call run_qemu_debug) &
	sleep 1
	$(GDB) $(OUT_ELF) \
	  -ex 'target remote localhost:1234' \
	  -ex 'b rust_entry' \
	  -ex 'continue' \
	  -ex 'disp /16i $$pc'

debug_no_attach: build
	$(call run_qemu_debug)

clippy:
ifeq ($(origin ARCH), command line)
	$(call cargo_clippy,--target $(TARGET))
else
	$(call cargo_clippy)
endif

doc:
	$(call cargo_doc)

doc_check_missing:
	$(call cargo_doc)

fmt:
	cargo fmt --all

fmt_c:
	@clang-format --style=file -i $(shell find ulib/ruxlibc -iname '*.c' -o -iname '*.h')

test:
	$(call app_test)

unittest:
	$(call unit_test)

unittest_no_fail_fast:
	$(call unit_test,--no-fail-fast)

disk_img:
ifneq ($(wildcard $(DISK_IMG)),)
	@printf "$(YELLOW_C)warning$(END_C): disk image \"$(DISK_IMG)\" already exists!\n"
else
	$(call make_disk_image,fat32,$(DISK_IMG))
endif

clean: clean_c clean_musl
	rm -rf $(APP)/*.bin $(APP)/*.elf
	cargo clean

clean_c::
	rm -rf ulib/ruxlibc/build_*
	rm -rf $(app-objs)

clean_musl:
	rm -rf ulib/ruxmusl/build_*
	rm -rf ulib/ruxmusl/install

.PHONY: all build disasm run justrun debug clippy fmt fmt_c test test_no_fail_fast clean clean_c clean_musl doc disk_image debug_no_attach
