/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Init
//!
//! firstly, a driver registers itself to get its index.
//! next, the driver registers all devices it found to get their indices.
//!
//! Read
//!
//! when a device receives data, it will cause a irq.
//! then the driver sends the data to tty layer using their indices.
//! finally, kernel will get the data using the device's name.  
//!
//! Write
//!
//! kernel writes data to a device using its name.

#![no_std]

extern crate alloc;

mod buffer;
mod constant;
mod driver;
mod ldisc;
mod tty;

use driver::get_driver_by_index;

pub use driver::{register_device, register_driver, TtyDriverOps};
pub use tty::{get_all_device_names, get_device_by_name};

/// called by driver when irq, to send data from hardware.
pub fn tty_receive_buf(driver_index: usize, device_index: usize, buf: &[u8]) {
    // check the validation of index
    if let Some(driver) = get_driver_by_index(driver_index) {
        if let Some(tty) = driver.get_device_by_index(device_index) {
            tty.ldisc().receive_buf(tty.clone(), buf);
        }
    }
}

/// called by kernel to read a tty device.
pub fn tty_read(buf: &mut [u8], dev_name: &str) -> usize {
    if let Some(tty) = get_device_by_name(dev_name) {
        tty.ldisc().read(buf)
    } else {
        0
    }
}

/// called by kernel to write a tty device.
pub fn tty_write(buf: &[u8], dev_name: &str) -> usize {
    if let Some(tty) = get_device_by_name(dev_name) {
        tty.ldisc().write(tty.clone(), buf)
    } else {
        0
    }
}

/// init
pub fn init() {
    driver::init();
    tty::init();
}
