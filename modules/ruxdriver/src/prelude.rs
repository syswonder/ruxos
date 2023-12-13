/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Device driver prelude that includes some traits and types.

pub use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

#[cfg(feature = "_9p")]
pub use {crate::structs::Ax9pDevice, driver_9p::_9pDriverOps};
#[cfg(feature = "block")]
pub use {crate::structs::AxBlockDevice, driver_block::BlockDriverOps};
#[cfg(feature = "display")]
pub use {crate::structs::AxDisplayDevice, driver_display::DisplayDriverOps};
#[cfg(feature = "net")]
pub use {crate::structs::AxNetDevice, driver_net::NetDriverOps};
