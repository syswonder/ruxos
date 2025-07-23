/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Common traits and types for block storage device drivers (i.e. disk).

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(const_trait_impl)]

#[doc(no_inline)]
pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

/// Operations that require a console device driver to implement.
pub trait ConsoleDriverOps: BaseDriverOps {
    /// Writes a single byte to the console.
    fn putchar(&mut self, c: u8);
    /// Reads a single byte from the console.
    fn getchar(&mut self) -> Option<u8>;
    /// Acknowledge an interrupt from the console.
    fn ack_interrupt(&mut self) -> DevResult<bool>;
}
