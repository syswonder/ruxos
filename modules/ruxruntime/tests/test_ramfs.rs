/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![cfg(feature = "myfs")]

mod test_common;

use std::sync::Arc;

use axfs_ramfs::RamFileSystem;
use axfs_vfs::VfsOps;
use axio::{Result, Write};
use driver_block::ramdisk::RamDisk;
use ruxdriver::AxDeviceContainer;
use ruxfs::api as fs;
use ruxfs::MyFileSystemIf;

struct MyFileSystemIfImpl;

#[crate_interface::impl_interface]
impl MyFileSystemIf for MyFileSystemIfImpl {
    fn new_myfs() -> Arc<dyn VfsOps> {
        Arc::new(RamFileSystem::new())
    }
}

fn create_init_files() -> Result<()> {
    fs::write(&fs::absolute_path("./short.txt")?, "Rust is cool!\n")?;
    let mut file = fs::File::create_new(&fs::absolute_path("/long.txt")?)?;
    for _ in 0..100 {
        file.write_fmt(format_args!("Rust is cool!\n"))?;
    }

    fs::create_dir(&fs::absolute_path("very-long-dir-name")?)?;
    fs::write(
        &fs::absolute_path("very-long-dir-name/very-long-file-name.txt")?,
        "Rust is cool!\n",
    )?;

    fs::create_dir(&fs::absolute_path("very")?)?;
    fs::create_dir(&fs::absolute_path("//very/long")?)?;
    fs::create_dir(&fs::absolute_path("/./very/long/path")?)?;
    fs::write(
        &fs::absolute_path(".//very/long/path/test.txt")?,
        "Rust is cool!\n",
    )?;
    Ok(())
}

#[test]
fn test_ramfs() {
    println!("Testing ramfs ...");

    ruxtask::init_scheduler(); // call this to use `axsync::Mutex`.
                               // By default, mount_points[0] will be rootfs

    let mut mount_points: Vec<ruxfs::root::MountPoint> = Vec::new();
    // setup and initialize blkfs as one mountpoint for rootfs
    mount_points.push(ruxfs::init_tempfs());
    ruxfs::prepare_commonfs(&mut mount_points);

    // setup and initialize rootfs
    ruxfs::init_filesystems(mount_points);

    if let Err(e) = create_init_files() {
        log::warn!("failed to create init files: {:?}", e);
    }

    test_common::test_all();
}
