/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Platform-specific constants and parameters for [Ruxos].
//!
//! Currently supported platforms can be found in the [platforms] directory of
//! the [Ruxos] root.
//!
//! [Ruxos]: https://github.com/syswonder/ruxos
//! [platforms]: https://github.com/syswonder/ruxos/tree/main/platforms

#![no_std]

#[rustfmt::skip]
mod config {
    include!(concat!(env!("OUT_DIR"), "/config.rs"));
}

pub use config::*;

/// End address of the whole physical memory.
pub const PHYS_MEMORY_END: usize = PHYS_MEMORY_BASE + PHYS_MEMORY_SIZE;
