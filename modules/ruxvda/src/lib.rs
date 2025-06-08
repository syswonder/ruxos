/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! dev fuse
#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_auto_cfg)]

// #[macro_use]
extern crate log;
extern crate alloc;

pub mod fs;

// use alloc::vec::Vec;
use alloc::string::String;
use alloc::sync::Arc;
use log::*;
use ruxdriver::{prelude::*, AxDeviceContainer};
use ruxfs::MountPoint;

pub fn init_vdafs(mut vda_devs: AxDeviceContainer<AxBlockDevice>) -> MountPoint {
    info!("Initialize VDA filesystem...");

    let vda = vda_devs.take_one().expect("No VDA device found!");
    info!("  use VDA device 0: {:?}", vda.device_name());

    // let vda_driver = self::drv::DrvVdaOps::new(vda);
    let vda_fs = self::fs::VdaFileSystem::new(vda);

    MountPoint::new(String::from("/vda1"), Arc::new(vda_fs))
}