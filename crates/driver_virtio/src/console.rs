/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::as_dev_err;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
use driver_console::ConsoleDriverOps;
use virtio_drivers::{device::console::VirtIOConsole as InnerDev, transport::Transport, Hal};

/// VirtIO console device
pub struct VirtIoConsoleDev<H: Hal, T: Transport> {
    inner: InnerDev<'static, H, T>,
}

unsafe impl<H: Hal, T: Transport> Send for VirtIoConsoleDev<H, T> {}
unsafe impl<H: Hal, T: Transport> Sync for VirtIoConsoleDev<H, T> {}

impl<H: Hal, T: Transport> VirtIoConsoleDev<H, T> {
    /// Creates a new driver instance and initializes the device, or returns
    /// an error if any step fails.
    pub fn try_new(transport: T) -> DevResult<Self> {
        Ok(Self {
            inner: InnerDev::new(transport).map_err(as_dev_err)?,
        })
    }
}

impl<H: Hal, T: Transport> BaseDriverOps for VirtIoConsoleDev<H, T> {
    fn device_name(&self) -> &str {
        "virtio-console"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Char
    }
}

impl<H: Hal, T: Transport> ConsoleDriverOps for VirtIoConsoleDev<H, T> {
    fn putchar(&mut self, c: u8) {
        self.inner
            .send(c)
            .expect("VirtConsole: failed to send char");
    }

    fn getchar(&mut self) -> Option<u8> {
        self.inner
            .recv(true)
            .expect("VirtConsole: failed to recv char")
    }

    fn ack_interrupt(&mut self) -> DevResult<bool> {
        self.inner.ack_interrupt().map_err(as_dev_err)
    }
}
