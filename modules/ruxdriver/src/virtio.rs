/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! A driver for VirtIO devices.

use crate::{drivers::DriverProbe, AxDeviceEnum};
use cfg_if::cfg_if;
use core::marker::PhantomData;
use driver_common::{BaseDriverOps, DevResult, DeviceType};
#[cfg(bus = "mmio")]
use ruxhal::mem::phys_to_virt;
#[cfg(any(
    feature = "virtio-net",
    feature = "virtio-blk",
    feature = "virtio-gpu",
    feature = "virtio-9p",
    feature = "pci"
))]
use ruxhal::virtio::virtio_hal::VirtIoHalImpl;

cfg_if! {
    if #[cfg(bus = "pci")] {
        use driver_pci::{PciRoot, DeviceFunction, DeviceFunctionInfo};
        type VirtIoTransport = driver_virtio::PciTransport;
    } else if #[cfg(bus =  "mmio")] {
        type VirtIoTransport = driver_virtio::MmioTransport;
    }
}

/// A trait for VirtIO device meta information.
pub trait VirtIoDevMeta {
    /// The device type of the VirtIO device.
    const DEVICE_TYPE: DeviceType;

    /// The device type of the VirtIO device.
    type Device: BaseDriverOps;
    /// The driver for the VirtIO device.
    type Driver = VirtIoDriver<Self>;

    /// Try to create a new instance of the VirtIO device.
    fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum>;
}

cfg_if! {
    if #[cfg(net_dev = "virtio-net")] {
        /// A VirtIO network device.
        pub struct VirtIoNet;

        impl VirtIoDevMeta for VirtIoNet {
            const DEVICE_TYPE: DeviceType = DeviceType::Net;
            type Device = driver_virtio::VirtIoNetDev<VirtIoHalImpl, VirtIoTransport, 64>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_net(Self::Device::try_new(transport)?))
            }
        }
    }
}

cfg_if! {
    if #[cfg(block_dev = "virtio-blk")] {
        /// A VirtIO block device.
        pub struct VirtIoBlk;

        impl VirtIoDevMeta for VirtIoBlk {
            const DEVICE_TYPE: DeviceType = DeviceType::Block;
            type Device = driver_virtio::VirtIoBlkDev<VirtIoHalImpl, VirtIoTransport>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_block(Self::Device::try_new(transport)?))
            }
        }
    }
}

cfg_if! {
    if #[cfg(display_dev = "virtio-gpu")] {
        /// A VirtIO GPU device.
        pub struct VirtIoGpu;

        impl VirtIoDevMeta for VirtIoGpu {
            const DEVICE_TYPE: DeviceType = DeviceType::Display;
            type Device = driver_virtio::VirtIoGpuDev<VirtIoHalImpl, VirtIoTransport>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_display(Self::Device::try_new(transport)?))
            }
        }
    }
}

cfg_if! {
    if #[cfg(_9p_dev = "virtio-9p")] {
        /// A VirtIO 9P device.
        pub struct VirtIo9p;

        impl VirtIoDevMeta for VirtIo9p {
            const DEVICE_TYPE: DeviceType = DeviceType::_9P;
            type Device = driver_virtio::VirtIo9pDev<VirtIoHalImpl, VirtIoTransport>;

            fn try_new(transport: VirtIoTransport) -> DevResult<AxDeviceEnum> {
                Ok(AxDeviceEnum::from_9p(Self::Device::try_new(transport)?))
            }
        }
    }
}

/// A common driver for all VirtIO devices that implements DriverProbe.
pub struct VirtIoDriver<D: VirtIoDevMeta + ?Sized>(PhantomData<D>);

impl<D: VirtIoDevMeta> DriverProbe for VirtIoDriver<D> {
    #[cfg(bus = "mmio")]
    fn probe_mmio(mmio_base: usize, mmio_size: usize) -> Option<AxDeviceEnum> {
        let base_vaddr = phys_to_virt(mmio_base.into());
        if let Some((ty, transport)) =
            driver_virtio::probe_mmio_device(base_vaddr.as_mut_ptr(), mmio_size)
        {
            if ty == D::DEVICE_TYPE {
                match D::try_new(transport) {
                    Ok(dev) => return Some(dev),
                    Err(e) => {
                        warn!(
                            "failed to initialize MMIO device at [PA:{:#x}, PA:{:#x}): {:?}",
                            mmio_base,
                            mmio_base + mmio_size,
                            e
                        );
                        return None;
                    }
                }
            }
        }
        None
    }

    #[cfg(bus = "pci")]
    fn probe_pci(
        root: &mut PciRoot,
        bdf: DeviceFunction,
        dev_info: &DeviceFunctionInfo,
    ) -> Option<AxDeviceEnum> {
        if dev_info.vendor_id != 0x1af4 {
            return None;
        }
        match (D::DEVICE_TYPE, dev_info.device_id) {
            (DeviceType::Net, 0x1000) | (DeviceType::Net, 0x1040) => {}
            (DeviceType::Block, 0x1001) | (DeviceType::Block, 0x1041) => {}
            (DeviceType::Display, 0x1050) => {}
            (DeviceType::_9P, 0x1009) => {}
            _ => return None,
        }

        if let Some((ty, transport)) =
            driver_virtio::probe_pci_device::<VirtIoHalImpl>(root, bdf, dev_info)
        {
            if ty == D::DEVICE_TYPE {
                match D::try_new(transport) {
                    Ok(dev) => return Some(dev),
                    Err(e) => {
                        warn!(
                            "failed to initialize PCI device at {}({}): {:?}",
                            bdf, dev_info, e
                        );
                        return None;
                    }
                }
            }
        }
        None
    }
}
