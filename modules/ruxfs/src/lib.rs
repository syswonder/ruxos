/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos](https://github.com/syswonder/ruxos) filesystem module.
//!
//! It provides unified filesystem operations for various filesystems.
//!
//! # Cargo Features
//!
//! - `fatfs`: Use [FAT] as the main filesystem and mount it on `/`. This feature
//!    is **enabled** by default.
//! - `devfs`: Mount [`axfs_devfs::DeviceFileSystem`] on `/dev`. This feature is
//!    **enabled** by default.
//! - `ramfs`: Mount [`axfs_ramfs::RamFileSystem`] on `/tmp`. This feature is
//!    **enabled** by default.
//! - `myfs`: Allow users to define their custom filesystems to override the
//!    default. In this case, [`MyFileSystemIf`] is required to be implemented
//!    to create and initialize other filesystems. This feature is **disabled** by
//!    by default, but it will override other filesystem selection features if
//!    both are enabled.
//!
//! [FAT]: https://en.wikipedia.org/wiki/File_Allocation_Table
//! [`MyFileSystemIf`]: fops::MyFileSystemIf

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;
extern crate alloc;

mod dev;
mod fs;
mod mounts;
pub mod root;

#[cfg(feature = "alloc")]
mod arch;

pub mod api;
pub mod fops;

use alloc::vec::Vec;

use ruxdriver::{prelude::*, AxDeviceContainer};

cfg_if::cfg_if! {
    if #[cfg(feature = "myfs")] {
    } else if #[cfg(feature = "fatfs")] {
        use lazy_init::LazyInit;
        use alloc::sync::Arc;
    }
}

pub use root::MountPoint;

/// Initialize an empty filesystems by ramfs.
#[cfg(not(any(feature = "blkfs", feature = "virtio-9p", feature = "net-9p")))]
pub fn init_tempfs() -> MountPoint {
    MountPoint::new("/", mounts::ramfs())
}

/// Initializes filesystems by block devices.
pub fn init_blkfs(mut blk_devs: AxDeviceContainer<AxBlockDevice>) -> MountPoint {
    info!("Initialize filesystems...");

    let dev = blk_devs.take_one().expect("No block device found!");
    info!("  use block device 0: {:?}", dev.device_name());

    let disk = self::dev::Disk::new(dev);
    cfg_if::cfg_if! {
        if #[cfg(feature = "myfs")] { // override the default filesystem
            let blk_fs = fs::myfs::new_myfs(disk);
        } else if #[cfg(feature = "fatfs")] {
            static FAT_FS: LazyInit<Arc<fs::fatfs::FatFileSystem>> = LazyInit::new();
            FAT_FS.init_by(Arc::new(fs::fatfs::FatFileSystem::new(disk)));
            FAT_FS.init();
            let blk_fs = FAT_FS.clone();
        }
    }

    MountPoint::new("/", blk_fs)
}

/// Initializes common filesystems.
pub fn prepare_commonfs(mount_points: &mut Vec<self::root::MountPoint>) {
    #[cfg(feature = "devfs")]
    let mount_point = MountPoint::new("/dev", mounts::devfs());
    mount_points.push(mount_point);

    #[cfg(feature = "ramfs")]
    let mount_point = MountPoint::new("/tmp", mounts::ramfs());
    mount_points.push(mount_point);

    // Mount another ramfs as procfs
    #[cfg(feature = "procfs")]
    let mount_point = MountPoint::new("/proc", mounts::procfs().unwrap());
    mount_points.push(mount_point);

    // Mount another ramfs as sysfs
    #[cfg(feature = "sysfs")]
    let mount_point = MountPoint::new("/sys", mounts::sysfs().unwrap());
    mount_points.push(mount_point);

    // Mount another ramfs as etcfs
    #[cfg(feature = "etcfs")]
    let mount_point = MountPoint::new("/etc", mounts::etcfs().unwrap());
    mount_points.push(mount_point);
}

/// Initializes root filesystems.
pub fn init_filesystems(mount_points: Vec<self::root::MountPoint>) {
    self::root::init_rootfs(mount_points);
}
