/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Common traits and types for 9P device drivers (i.e. 9P2000.L, 9P2000.U).

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(const_trait_impl)]

#[doc(no_inline)]
pub use driver_common::BaseDriverOps;

/// Operations that require a 9p driver to implement.
pub trait _9pDriverOps: BaseDriverOps {
    /// initialize self(e.g. setup TCP connection)
    fn init(&self) -> Result<(), u8>;

    /// send bytes of inputs as request and receive  get answer in outputs
    fn send_with_recv(&mut self, inputs: &[u8], outputs: &mut [u8]) -> Result<u32, u8>; // Ok(length)/Err()
}
