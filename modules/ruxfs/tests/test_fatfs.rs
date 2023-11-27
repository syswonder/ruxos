/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![cfg(not(feature = "myfs"))]

mod test_common;

use driver_block::ramdisk::RamDisk;
use ruxdriver::AxDeviceContainer;

const IMG_PATH: &str = "resources/fat16.img";

fn make_disk() -> std::io::Result<RamDisk> {
    let path = std::env::current_dir()?.join(IMG_PATH);
    println!("Loading disk image from {:?} ...", path);
    let data = std::fs::read(path)?;
    println!("size = {} bytes", data.len());
    Ok(RamDisk::from(&data))
}

#[test]
fn test_fatfs() {
    println!("Testing fatfs with ramdisk ...");

    let disk = make_disk().expect("failed to load disk image");
    ruxtask::init_scheduler(); // call this to use `axsync::Mutex`.
                               // By default, mount_points[0] will be rootfs
    let mut mount_points: Vec<ruxfs::MountPoint> = Vec::new();
    // setup and initialize blkfs as one mountpoint for rootfs
    mount_points.push(ruxfs::init_blkfs(AxDeviceContainer::from_one(disk)));
    ruxfs::prepare_commonfs(&mut mount_points);

    // setup and initialize rootfs
    ruxfs::init_filesystems(mount_points);

    test_common::test_all();
}
