/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! TODO: generate registered drivers in `for_each_drivers!` automatically.

#![allow(unused_macros)]

macro_rules! register_net_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxNetDevice = $device_type;
    };
}

macro_rules! register_block_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxBlockDevice = $device_type;
    };
}

macro_rules! register_display_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type AxDisplayDevice = $device_type;
    };
}

macro_rules! register_9p_driver {
    ($driver_type:ty, $device_type:ty) => {
        /// The unified type of the NIC devices.
        #[cfg(not(feature = "dyn"))]
        pub type Ax9pDevice = $device_type;
    };
}

macro_rules! for_each_drivers {
    (type $drv_type:ident, $code:block) => {{
        #[allow(unused_imports)]
        use crate::drivers::DriverProbe;
        #[cfg(feature = "virtio")]
        #[allow(unused_imports)]
        use crate::virtio::{self, VirtIoDevMeta};

        #[cfg(net_dev = "virtio-net")]
        {
            type $drv_type = <virtio::VirtIoNet as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(block_dev = "virtio-blk")]
        {
            type $drv_type = <virtio::VirtIoBlk as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(display_dev = "virtio-gpu")]
        {
            type $drv_type = <virtio::VirtIoGpu as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(_9p_dev = "virtio-9p")]
        {
            type $drv_type = <virtio::VirtIo9p as VirtIoDevMeta>::Driver;
            $code
        }
        #[cfg(block_dev = "ramdisk")]
        {
            type $drv_type = crate::drivers::RamDiskDriver;
            $code
        }
        #[cfg(block_dev = "bcm2835-sdhci")]
        {
            type $drv_type = crate::drivers::BcmSdhciDriver;
            $code
        }
        #[cfg(net_dev = "ixgbe")]
        {
            type $drv_type = crate::drivers::IxgbeDriver;
            $code
        }
    }};
}
