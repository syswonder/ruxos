/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg_attr(feature = "dyn", path = "dyn.rs")]
#[cfg_attr(not(feature = "dyn"), path = "static.rs")]
mod imp;

use driver_common::{BaseDriverOps, DeviceType};

pub use imp::*;

/// A unified enum that represents different categories of devices.
#[allow(clippy::large_enum_variant)]
pub enum AxDeviceEnum {
    /// Network card device.
    #[cfg(feature = "net")]
    Net(AxNetDevice),
    /// Block storage device.
    #[cfg(feature = "block")]
    Block(AxBlockDevice),
    /// Graphic display device.
    #[cfg(feature = "display")]
    Display(AxDisplayDevice),
    /// Random number generator device.
    #[cfg(feature = "rng")]
    Rng(AxRngDevice),
    /// Plan-9 protocol device.
    #[cfg(feature = "_9p")]
    _9P(Ax9pDevice),
}

impl BaseDriverOps for AxDeviceEnum {
    #[inline]
    #[allow(unreachable_patterns)]
    fn device_type(&self) -> DeviceType {
        match self {
            #[cfg(feature = "net")]
            Self::Net(_) => DeviceType::Net,
            #[cfg(feature = "block")]
            Self::Block(_) => DeviceType::Block,
            #[cfg(feature = "display")]
            Self::Display(_) => DeviceType::Display,
            #[cfg(feature = "rng")]
            Self::Rng(_) => DeviceType::Rng,
            #[cfg(feature = "_9p")]
            Self::_9P(_) => DeviceType::_9P,
            _ => unreachable!(),
        }
    }

    #[inline]
    #[allow(unreachable_patterns)]
    fn device_name(&self) -> &str {
        match self {
            #[cfg(feature = "net")]
            Self::Net(dev) => dev.device_name(),
            #[cfg(feature = "block")]
            Self::Block(dev) => dev.device_name(),
            #[cfg(feature = "display")]
            Self::Display(dev) => dev.device_name(),
            #[cfg(feature = "rng")]
            Self::Rng(dev) => dev.device_name(),
            #[cfg(feature = "_9p")]
            Self::_9P(dev) => dev.device_name(),
            _ => unreachable!(),
        }
    }
}
