/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Common traits and types for random number generator device drivers.

#![no_std]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

/// The information of the graphics device.
#[derive(Debug, Clone, Copy)]
pub struct RngInfo {}

/// Operations that require a graphics device driver to implement.
pub trait RngDriverOps: BaseDriverOps {
    /// Get the random number generator information.
    fn info(&self) -> RngInfo;
    /// Request random bytes from the device.
    fn request_entropy(&mut self, dst: &mut [u8]) -> DevResult<usize>;
}
