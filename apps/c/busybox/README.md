# busybox

## Quick Start

1. Compile `busybox` or get its ELF binary (using Musl), then copy to `rootfs/bin`.

2. Copy the Musl dyanmic linker to `rootfs/lib`.

3. modify `axbuild.mk`, like:

```makefile
app-objs=main.o

ARGS = /bin/busybox,ls
ENVS = 
V9P_PATH=${APP}/rootfs
```

4. Run

```sh
# in the RuxOS main directory.
make run ARCH=aarch64 A=apps/c/busybox V9P=y MUSL=y 
```
