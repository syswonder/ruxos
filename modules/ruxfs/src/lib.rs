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
//! - `fatfs`: Use [FAT] as the main filesystem and mount it on `/`. Requires
//!    `blkfs` to be enabled.
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

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

#[macro_use]
extern crate log;
extern crate alloc;

#[cfg(feature = "alloc")]
mod arch;
mod fs;
mod mounts;

pub mod api;
#[cfg(feature = "blkfs")]
pub mod dev;
pub mod fops;
pub mod root;

// Re-export `axfs_vfs` path types.

/// Canonicalized absolute path type. Requirements:
///
/// - Starting with `/`
/// - No `.` or `..` components
/// - No redundant or tailing `/`
/// - Valid examples: `/`, `/root/foo/bar`
pub type AbsPath<'a> = axfs_vfs::AbsPath<'a>;

/// Canonicalized relative path type. Requirements:
///
/// - No starting `/`
/// - No `.` components
/// - No redundant or tailing `/`
/// - Possibly starts with `..`
/// - Valid examples: ` `, `..`, `../b`, `../..`
pub type RelPath<'a> = axfs_vfs::RelPath<'a>;

#[cfg(feature = "myfs")]
pub use fs::myfs::MyFileSystemIf;

use alloc::vec::Vec;

#[cfg(feature = "blkfs")]
use ruxdriver::{prelude::*, AxDeviceContainer};

cfg_if::cfg_if! {
    if #[cfg(feature = "myfs")] {
    } else if #[cfg(feature = "fatfs")] {
        use lazy_init::LazyInit;
        use alloc::sync::Arc;
    // TODO: wait for CI support for ext4
    // } else if #[cfg(feature = "lwext4_rust")] {
    //     use lazy_init::LazyInit;
    //     use alloc::sync::Arc;
    } else if #[cfg(feature = "ext4_rs")] {
        use lazy_init::LazyInit;
        use alloc::sync::Arc;
    } else if #[cfg(feature = "another_ext4")] {
        use lazy_init::LazyInit;
        use alloc::sync::Arc;
    }
}

use root::MountPoint;

/// Initialize an empty filesystems by ramfs.
#[cfg(not(any(feature = "blkfs", feature = "virtio-9p", feature = "net-9p")))]
pub fn init_tempfs() -> MountPoint {
    MountPoint::new(AbsPath::new("/"), mounts::ramfs())
}

/// Initializes filesystems by block devices.
#[cfg(feature = "blkfs")]
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
        // TODO: wait for CI support for ext4
        // } else if #[cfg(feature = "lwext4_rust")] {
        //     static EXT4_FS: LazyInit<Arc<fs::lwext4_rust::Ext4FileSystem>> = LazyInit::new();
        //     EXT4_FS.init_by(Arc::new(fs::lwext4_rust::Ext4FileSystem::new(disk)));
        //     let blk_fs = EXT4_FS.clone();
        } else if #[cfg(feature = "ext4_rs")] {
            static EXT4_FS: LazyInit<Arc<fs::ext4_rs::Ext4FileSystem>> = LazyInit::new();
            EXT4_FS.init_by(Arc::new(fs::ext4_rs::Ext4FileSystem::new(disk)));
            let blk_fs = EXT4_FS.clone();
        } else if #[cfg(feature = "another_ext4")] {
            static EXT4_FS: LazyInit<Arc<fs::another_ext4::Ext4FileSystem>> = LazyInit::new();
            EXT4_FS.init_by(Arc::new(fs::another_ext4::Ext4FileSystem::new(disk)));
            let blk_fs = EXT4_FS.clone();
        } else {
            compile_error!("Please enable one of the block filesystems!");
        }
    }

    MountPoint::new(AbsPath::new("/"), blk_fs)
}

/// Initializes common filesystems.
pub fn prepare_commonfs(mount_points: &mut Vec<self::root::MountPoint>) {
    #[cfg(feature = "devfs")]
    let mount_point = MountPoint::new(AbsPath::new("/dev"), mounts::devfs());
    mount_points.push(mount_point);

    #[cfg(feature = "ramfs")]
    let mount_point = MountPoint::new(AbsPath::new("/tmp"), mounts::ramfs());
    mount_points.push(mount_point);

    // Mount another ramfs as procfs
    #[cfg(feature = "procfs")]
    let mount_point = MountPoint::new(AbsPath::new("/proc"), mounts::procfs().unwrap());
    mount_points.push(mount_point);

    // Mount another ramfs as sysfs
    #[cfg(feature = "sysfs")]
    let mount_point = MountPoint::new(AbsPath::new("/sys"), mounts::sysfs().unwrap());
    mount_points.push(mount_point);

    // Mount another ramfs as etcfs
    #[cfg(feature = "etcfs")]
    let mount_point = MountPoint::new(AbsPath::new("/etc"), mounts::etcfs().unwrap());
    mount_points.push(mount_point);
}

/// Initializes root filesystems.
pub fn init_filesystems(mount_points: Vec<self::root::MountPoint>) {
    self::fops::init_rootfs(mount_points);
}
