#! /bin/sh

set -e

opensbi_release="https://github.com/riscv-software-src/opensbi/releases/download/v1.4/opensbi-1.4-rv-bin.tar.xz"
filename=opensbi.tar.xz

prevd=$(pwd | xargs realpath)
tempd=$(mktemp -d --suffix=-ruxos)
echo "created temp dir at $tempd"

cleanup() {
    rm -r $tempd
    echo "cleaned temp dir"
}
trap cleanup EXIT

cd $tempd

wget $opensbi_release -P . -O $filename
tar -xf $filename -C .
rm $filename

cd $(ls)
mv -vn share/opensbi/lp64/generic/firmware/fw_dynamic.bin $prevd/fw_dynamic.bin

cd $prevd
