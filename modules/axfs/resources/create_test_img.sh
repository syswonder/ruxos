#!/bin/bash

# From https://github.com/rafalh/rust-fatfs/blob/master/scripts/create-test-img.sh

CUR_DIR=`dirname $0`

echo $OUT_DIR

create_test_img() {
	local name=$1
	local blkcount=$2
	local fatSize=$3
	dd if=/dev/zero of="$name" bs=1024 count=$blkcount
	mkfs.vfat -s 1 -F $fatSize -n "Test!" -i 12345678 "$name"
	mkdir -p mnt
	sudo mount -o loop "$name" mnt -o rw,uid=$USER,gid=$USER
	for i in $(seq 1 1000); do
	  echo "Rust is cool!" >>"mnt/long.txt"
	done
	echo "Rust is cool!" >>"mnt/short.txt"
	mkdir -p "mnt/very/long/path"
	echo "Rust is cool!" >>"mnt/very/long/path/test.txt"
	mkdir -p "mnt/very-long-dir-name"
	echo "Rust is cool!" >>"mnt/very-long-dir-name/very-long-file-name.txt"
	mkdir -p "mnt/nginx/logs"
	echo "" >> "mnt/nginx/logs/error.log"
	mkdir -p "mnt/nginx/conf"
	cp "/nginx/conf/nginx.conf" "mnt/nginx/conf/nginx.conf"
	cp "/nginx/conf/mime.types" "mnt/nginx/conf/mime.types"
	cp "/home/oslab/Desktop/nginx-1.24.0/output.txt" "mnt/testread.txt"

	sudo umount mnt
}

create_test_img "$CUR_DIR/fat16.img" 2500 16
create_test_img "$CUR_DIR/fat32.img" 40000 32
rm -f /home/oslab/Desktop/rukos/disk.img
cp fat32.img /home/oslab/Desktop/rukos/disk.img
