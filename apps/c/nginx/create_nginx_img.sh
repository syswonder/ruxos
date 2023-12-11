#!/bin/bash

# From https://github.com/rafalh/rust-fatfs/blob/master/scripts/create-test-img.sh
CUR_DIR=$(dirname $0)

echo $OUT_DIR

if [ $# -lt 1 ]; then
  CONF="$CUR_DIR/nginx.conf"
  echo "not us 9p"
elif [ "$1" == "9p" ]; then
  CONF="$CUR_DIR/nginx_9p.conf"
  echo "use 9p"
else
  CONF="$CUR_DIR/nginx.conf"
  echo "not use 9p"
fi

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
	mkdir -p "mnt/nginx/conf"
	cp "$CONF" "mnt/nginx/conf/nginx.conf"
	cp "$CUR_DIR/mime.types" "mnt/nginx/conf/mime.types"
	mkdir -p "mnt/html"
	cp -r "$CUR_DIR/html" "mnt/"
	sudo umount mnt
}

create_test_img "$CUR_DIR/fat32.img" 40000 32
echo $CUR_DIR
cat "$CUR_DIR/html/community.md"
echo "nginx create disk"
rm -f disk.img
cp $CUR_DIR/fat32.img disk.img
