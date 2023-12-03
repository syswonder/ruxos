#!/bin/bash

# From https://github.com/rafalh/rust-fatfs/blob/master/scripts/create-test-img.sh

CUR_DIR=`dirname $0`

echo $OUT_DIR
current_dir=$(pwd)
parent_dir1=$(dirname "$current_dir")
parent_dir2=$(dirname "$parent_dir1")
parent_dir3=$(dirname "$parent_dir2")

create_test_img() {
	local name=$1
	local blkcount=$2
	local fatSize=$3
	dd if=/dev/zero of="$name" bs=1024 count=$blkcount
	mkfs.vfat -s 1 -F $fatSize -n "Test!" -i 12345678 "$name"
	mkdir -p mnt
	sudo mount -o loop "$name" mnt -o rw,uid=$USER,gid=$USER
	mkdir -p "mnt/nginx/logs"
	mkdir -p "mnt/etc"
	echo "" >> "mnt/etc/localtime"
	echo "root:x:0:0:root:/root:/bin/bash" >> "mnt/etc/passwd"
	echo "root:x:0:" >> "mnt/etc/group"
#	echo "" >> "mnt/nginx/logs/error.log"
	mkdir -p "mnt/nginx/conf"
	cp "$CUR_DIR/nginx.conf" "mnt/nginx/conf/nginx.conf"
	cp "$CUR_DIR/mime.types" "mnt/nginx/conf/mime.types"
	mkdir -p "mnt/html"
	cp -r "$CUR_DIR/html" "mnt/"
	sudo umount mnt
}

create_test_img "$CUR_DIR/fat16.img" 2500 16
create_test_img "$CUR_DIR/fat32.img" 40000 32
echo $parent_dir3
rm -f $parent_dir3/disk.img
cp fat32.img $parent_dir3/disk.img
