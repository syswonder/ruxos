/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [RuxOS](https://github.com/syswonder/ruxos) 9p module.
//!
//! Implement `net-9p` and `virtio-9p`
//! Shouldn't mount file or directory with the same path as file or directory in 9P host
//! Or lookup() will only return mounted filesystem node.

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]
#![feature(ip_in_core)]
#![cfg(any(feature = "virtio-9p", feature = "net-9p"))]

#[doc(no_inline)]
extern crate alloc;
extern crate log;

mod drv;
mod fs;
#[cfg(feature = "net-9p")]
mod netdev;

use alloc::sync::Arc;
use log::*;
use ruxfs::MountPoint;
use spin::RwLock;

#[cfg(feature = "virtio-9p")]
use ruxdriver::{prelude::*, AxDeviceContainer};
#[cfg(feature = "net-9p")]
use {
    alloc::{boxed::Box, vec::Vec},
    core::option::Option::Some,
    driver_common::BaseDriverOps,
};

#[cfg(feature = "virtio-9p")]
/// Initializes filesystems by 9pfs devices.
pub fn init_virtio_9pfs(
    mut v9p_devs: AxDeviceContainer<Ax9pDevice>,
    aname: &str,
    protocol: &str,
) -> MountPoint {
    info!("Initialize virtio 9pfs...");

    let v9p = v9p_devs.take_one().expect("No 9pfs device found!");
    info!("  use 9pfs device 0: {:?}", v9p.device_name());

    let v9p_driver = self::drv::Drv9pOps::new(v9p);
    let v9p_fs = self::fs::_9pFileSystem::new(Arc::new(RwLock::new(v9p_driver)), aname, protocol);

    MountPoint::new("/v9fs", Arc::new(v9p_fs))
}

#[cfg(feature = "net-9p")]
/// Initializes filesystems by 9pfs devices.
pub fn init_net_9pfs(ip_port: &str, aname: &str, protocol: &str) -> MountPoint {
    info!("Initialize net 9pfs...");

    let net9p = match parse_address(ip_port) {
        Some((ip, port)) => self::netdev::Net9pDev::new(&ip, port),
        None => self::netdev::Net9pDev::new(&[127, 0, 0, 1], 564_u16), // use 127.0.0.1:564 in defealt
    };
    info!("use 9pfs device 0: {:?}", net9p.device_name());

    // Enabling `dyn` feature in ruxdriver, pub type Ax9pDevice = Box<dyn _9pDriverOps>;
    // TODO: consider a more elegant implement.
    let net9p_driver = self::drv::Drv9pOps::new(Box::new(net9p));
    let n9p_fs = self::fs::_9pFileSystem::new(Arc::new(RwLock::new(net9p_driver)), aname, protocol);

    MountPoint::new("/n9fs", Arc::new(n9p_fs))
}

#[cfg(feature = "net-9p")]
fn parse_address(s: &str) -> Option<(Vec<u8>, u16)> {
    let mut parts = s.split(':');
    if let (Some(address), Some(port)) = (parts.next(), parts.next()) {
        let parsed_address: Result<Vec<u8>, _> =
            address.split('.').map(|part| part.parse::<u8>()).collect();

        if let Ok(address_parts) = parsed_address {
            if let Ok(port) = port.parse::<u16>() {
                return Some((address_parts, port));
            }
        }
    }
    None
}
