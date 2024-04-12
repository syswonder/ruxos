app-objs=main.o

ARGS = /bin/hello
ENVS = 
V9P_PATH=${APP}/rootfs

# make run ARCH=aarch64 A=apps/c/dl V9P=y MUSL=y LOG=debug