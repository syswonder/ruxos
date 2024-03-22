#! /bin/sh

git submodule update --init --recursive --remote
export CROSS_COMPILE=riscv64-linux-musl-
make -C patches/opensbi PLATFORM=generic; make run APP=apps/c/helloworld ARCH=riscv64 FEATURES=musl,multitask MUSL=y LOG=info
