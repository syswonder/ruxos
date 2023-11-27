/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use driver_9p::_9pDriverOps;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use virtio_drivers::{device::v9p::VirtIO9p as InnerDev, transport::Transport, Hal};

/// The VirtIO 9p device driver.
pub struct VirtIo9pDev<H: Hal, T: Transport> {
    inner: InnerDev<H, T>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIo9pDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIo9pDev<H, T> {}

impl<H: Hal, T: Transport> VirtIo9pDev<H, T> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        Ok(Self {
            inner: InnerDev::new(transport).unwrap(),
        })
    }
}

impl<H: Hal, T: Transport> const BaseDriverOps for VirtIo9pDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-9p"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::_9P
    }
}

impl<H: Hal, T: Transport> _9pDriverOps for VirtIo9pDev<H, T> {
    // initialize self(e.g. setup TCP connection)
    fn init(&self) -> Result<(), u8> {
        Ok(())
    }

    // send bytes of inputs as request and receive  get answer in outputs
    fn send_with_recv(&mut self, inputs: &[u8], outputs: &mut [u8]) -> Result<u32, u8> {
        self.inner.request(inputs, outputs)
    }
}
