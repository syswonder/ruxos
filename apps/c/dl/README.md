# ELF loader

> Read the RuxOS Book for detail.

## Quick Start

1. Compile the C files with Musl in `rootfs/`.

```sh
cd rootfs/
musl-gcc libadd.c -shared -o lib/libadd.so
musl-gcc hello.c -Llib -ladd -o bin/hello
```

2. Copy the Musl dyanmic linker to `rootfs/lib`.

3. Run

Run with `ruxgo`:

```sh
# in apps/c/dl
ruxgo -b && ruxgo -r
```

Run with `make`

```sh
# in the RuxOS main directory.
make run ARCH=aarch64 A=apps/c/dl V9P=y MUSL=y LOG=debug
```
