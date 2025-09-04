/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

extern crate alloc;
use crate::as_dev_err;

use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_rng::{RngDriverOps, RngInfo};
use virtio_drivers::{device::rng::VirtIORng as InnerDev, transport::Transport, Hal};

/// The VirtIO RNG device driver.
pub struct VirtIoRngDev<H: Hal, T: Transport> {
    inner: InnerDev<H, T>,
    info: RngInfo,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoRngDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoRngDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoRngDev<H, T> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        let virtio = InnerDev::new(transport).unwrap();
        let info = RngInfo {};
        Ok(Self {
            inner: virtio,
            info,
        })
    }
}

impl<H: Hal, T: Transport> RngDriverOps for VirtIoRngDev<H, T> {
    fn info(&self) -> RngInfo {
        self.info
    }

    fn request_entropy(&mut self, dst: &mut [u8]) -> DevResult<usize> {
        match self.inner.request_entropy(dst) {
            Ok(size) => Ok(size),
            Err(e) => Err(as_dev_err(e)),
        }
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIoRngDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-rng"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Rng
    }
}
