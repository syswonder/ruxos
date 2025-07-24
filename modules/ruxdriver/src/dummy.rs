/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Dummy types used if no device of a certain category is selected.

#![allow(unused_imports)]
#![allow(dead_code)]

use super::prelude::*;
use alloc::sync::Arc;
use cfg_if::cfg_if;

cfg_if! {
    if #[cfg(net_dev = "dummy")] {
        use driver_net::{EthernetAddress, NetBuf, NetBufBox, NetBufPool, NetBufPtr};

        pub struct DummyNetDev;
        pub struct DummyNetDrvier;
        register_net_driver!(DummyNetDriver, DummyNetDev);

        impl BaseDriverOps for DummyNetDev {
            fn device_type(&self) -> DeviceType { DeviceType::Net }
            fn device_name(&self) -> &str { "dummy-net" }
        }

        impl NetDriverOps for DummyNetDev {
            fn mac_address(&self) -> EthernetAddress { unreachable!() }
            fn can_transmit(&self) -> bool { false }
            fn can_receive(&self) -> bool { false }
            fn rx_queue_size(&self) -> usize { 0 }
            fn tx_queue_size(&self) -> usize { 0 }
            fn fill_rx_buffers(&mut self, _: &Arc<NetBufPool>) -> DevResult { Err(DevError::Unsupported) }
            fn prepare_tx_buffer(&self, _: &mut NetBuf, _: usize) -> DevResult { Err(DevError::Unsupported) }
            fn recycle_rx_buffer(&mut self, _: NetBufPtr) -> DevResult { Err(DevError::Unsupported) }
            fn recycle_tx_buffers(&mut self) -> DevResult { Err(DevError::Unsupported) }
            fn transmit(&mut self, _: NetBufPtr) -> DevResult { Err(DevError::Unsupported) }
            fn receive(&mut self) -> DevResult<NetBufPtr> { Err(DevError::Unsupported) }
            fn alloc_tx_buffer(&mut self, _: usize) -> DevResult<NetBufPtr> { Err(DevError::Unsupported) }
        }
    }
}

cfg_if! {
    if #[cfg(block_dev = "dummy")] {
        pub struct DummyBlockDev;
        pub struct DummyBlockDriver;
        register_block_driver!(DummyBlockDriver, DummyBlockDev);

        impl BaseDriverOps for DummyBlockDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Block
            }
            fn device_name(&self) -> &str {
                "dummy-block"
            }
        }

        impl BlockDriverOps for DummyBlockDev {
            fn num_blocks(&self) -> u64 {
                0
            }
            fn block_size(&self) -> usize {
                0
            }
            fn read_block(&mut self, _: u64, _: &mut [u8]) -> DevResult {
                Err(DevError::Unsupported)
            }
            fn write_block(&mut self, _: u64, _: &[u8]) -> DevResult {
                Err(DevError::Unsupported)
            }
            fn flush(&mut self) -> DevResult {
                Err(DevError::Unsupported)
            }
        }
    }
}

cfg_if! {
    if #[cfg(display_dev = "dummy")] {
        pub struct DummyDisplayDev;
        pub struct DummyDisplayDriver;
        register_display_driver!(DummyDisplayDriver, DummyDisplayDev);

        impl BaseDriverOps for DummyDisplayDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Display
            }
            fn device_name(&self) -> &str {
                "dummy-display"
            }
        }

        impl DisplayDriverOps for DummyDisplayDev {
            fn info(&self) -> driver_display::DisplayInfo {
                unreachable!()
            }
            fn fb(&self) -> driver_display::FrameBuffer {
                unreachable!()
            }
            fn need_flush(&self) -> bool {
                false
            }
            fn flush(&mut self) -> DevResult {
                Err(DevError::Unsupported)
            }
        }
    }
}

cfg_if! {
    if #[cfg(rng_dev = "dummy")] {
        pub struct DummyRngDev;
        pub struct DummyRngDriver;
        register_rng_driver!(DummyRngDriver, DummyRngDev);

        impl BaseDriverOps for DummyRngDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::Rng
            }
            fn device_name(&self) -> &str {
                "dummy-rng"
            }
        }

        impl RngDriverOps for DummyRngDev {
            fn info(&self) -> driver_rng::RngInfo {
                unreachable!()
            }
            fn request_entropy(&mut self, _: &mut [u8]) -> DevResult<usize> {
                unreachable!()
            }
        }
    }
}

cfg_if! {
    if #[cfg(_9p_dev = "dummy")] {
        pub struct Dummy9pDev;
        pub struct Dummy9pDriver;
        register_9p_driver!(Dummy9pDriver, Dummy9pDev);

        impl BaseDriverOps for Dummy9pDev {
            fn device_type(&self) -> DeviceType {
                DeviceType::_9P
            }
            fn device_name(&self) -> &str {
                "dummy-9p"
            }
        }

        impl _9pDriverOps for Dummy9pDev {
            fn init(&self) -> Result<(), u8>{
                Err(0)
            }

            #[allow(unused_variables)]
            fn send_with_recv(&mut self, inputs: &[u8], outputs: &mut [u8]) -> Result<u32, u8>{
                Err(0)
            }
        }
    }
}
