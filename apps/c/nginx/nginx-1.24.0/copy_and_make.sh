#!/bin/bash

# 从 filename.txt 中读取最后一行的文件名
filename=$(tail -n 1 filename.txt)

# 构建源文件和目标文件的路径
source_file="/home/oslab/Desktop/musl/aarch64-linux-musl-cross/aarch64-linux-musl/include/$filename"
destination_file="/home/oslab/Desktop/rukos/ulib/axlibc/include/$filename"

# 使用 cp 命令复制文件
cp "$source_file" "$destination_file"

# 执行 make 命令
make
