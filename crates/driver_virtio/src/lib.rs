/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Wrappers of some devices in the [`virtio-drivers`][1] crate, that implement
//! traits in the [`driver_common`][2] series crates.
//!
//! Like the [`virtio-drivers`][1] crate, you must implement the [`VirtIoHal`]
//! trait (alias of [`virtio-drivers::Hal`][3]), to allocate DMA regions and
//! translate between physical addresses (as seen by devices) and virtual
//! addresses (as seen by your program).
//!
//! [1]: https://docs.rs/virtio-drivers/latest/virtio_drivers/
//! [2]: ../driver_common/index.html
//! [3]: https://docs.rs/virtio-drivers/latest/virtio_drivers/trait.Hal.html

#![no_std]
#![feature(const_trait_impl)]
#![feature(doc_auto_cfg)]

#[cfg(feature = "block")]
mod blk;
#[cfg(feature = "gpu")]
mod gpu;
#[cfg(feature = "net")]
mod net;
#[cfg(feature = "v9p")]
mod v9p;

#[cfg(feature = "block")]
pub use self::blk::VirtIoBlkDev;
#[cfg(feature = "gpu")]
pub use self::gpu::VirtIoGpuDev;
#[cfg(feature = "net")]
pub use self::net::VirtIoNetDev;
#[cfg(feature = "v9p")]
pub use self::v9p::VirtIo9pDev;

pub use virtio_drivers::transport::pci::bus as pci;
pub use virtio_drivers::transport::{mmio::MmioTransport, pci::PciTransport, Transport};
pub use virtio_drivers::{BufferDirection, Hal as VirtIoHal, PhysAddr};

use self::pci::{DeviceFunction, DeviceFunctionInfo, PciRoot};
use driver_common::{DevError, DeviceType};
use virtio_drivers::transport::DeviceType as VirtIoDevType;

/// Try to probe a VirtIO MMIO device from the given memory region.
///
/// If the device is recognized, returns the device type and a transport object
/// for later operations. Otherwise, returns [`None`].
pub fn probe_mmio_device(
    reg_base: *mut u8,
    _reg_size: usize,
) -> Option<(DeviceType, MmioTransport)> {
    use core::ptr::NonNull;
    use virtio_drivers::transport::mmio::VirtIOHeader;

    let header = NonNull::new(reg_base as *mut VirtIOHeader).unwrap();
    let transport = unsafe { MmioTransport::new(header) }.ok()?;
    let dev_type = as_dev_type(transport.device_type())?;
    Some((dev_type, transport))
}

/// Try to probe a VirtIO PCI device from the given PCI address.
///
/// If the device is recognized, returns the device type and a transport object
/// for later operations. Otherwise, returns [`None`].
pub fn probe_pci_device<H: VirtIoHal>(
    root: &mut PciRoot,
    bdf: DeviceFunction,
    dev_info: &DeviceFunctionInfo,
) -> Option<(DeviceType, PciTransport)> {
    use virtio_drivers::transport::pci::virtio_device_type;

    let dev_type = virtio_device_type(dev_info).and_then(as_dev_type)?;
    let transport = PciTransport::new::<H>(root, bdf).ok()?;
    Some((dev_type, transport))
}

const fn as_dev_type(t: VirtIoDevType) -> Option<DeviceType> {
    use VirtIoDevType::*;
    match t {
        Block => Some(DeviceType::Block),
        Network => Some(DeviceType::Net),
        GPU => Some(DeviceType::Display),
        _9P => Some(DeviceType::_9P),
        _ => None,
    }
}

#[allow(dead_code)]
const fn as_dev_err(e: virtio_drivers::Error) -> DevError {
    use virtio_drivers::Error::*;
    match e {
        QueueFull => DevError::BadState,
        NotReady => DevError::Again,
        WrongToken => DevError::BadState,
        AlreadyUsed => DevError::AlreadyExists,
        InvalidParam => DevError::InvalidParam,
        DmaError => DevError::NoMemory,
        IoError => DevError::Io,
        Unsupported => DevError::Unsupported,
        ConfigSpaceTooSmall => DevError::BadState,
        ConfigSpaceMissing => DevError::BadState,
        _ => DevError::BadState,
    }
}
