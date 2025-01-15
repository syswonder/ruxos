/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! virtio_console
use crate::mem::phys_to_virt;
use crate::virtio::virtio_hal::VirtIoHalImpl;
use driver_console::ConsoleDriverOps;
use driver_virtio::VirtIoConsoleDev;
use spinlock::SpinNoIrq;
const VIRTIO_CONSOLE_BASE: usize = ruxconfig::VIRTIO_CONSOLE_PADDR;
const VIRTIO_CONSOLE_REG: usize = 0x200;

#[cfg(all(feature = "irq", target_arch = "aarch64"))]
use crate::platform::irq::VIRTIO_CONSOLE_IRQ_NUM;

/// Store buffer size
const MEM_SIZE: usize = 4096;

#[cfg(feature = "irq")]
const BUFFER_SIZE: usize = 128;

#[cfg(feature = "irq")]
struct RxRingBuffer {
    buffer: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
    empty: bool,
}

/// The UART RxRingBuffer
#[cfg(feature = "irq")]
impl RxRingBuffer {
    /// Create a new ring buffer
    const fn new() -> Self {
        RxRingBuffer {
            buffer: [0_u8; BUFFER_SIZE],
            head: 0_usize,
            tail: 0_usize,
            empty: true,
        }
    }

    /// Push a byte into the buffer
    fn push(&mut self, n: u8) {
        if self.tail != self.head || self.empty {
            self.buffer[self.tail] = n;
            self.tail = (self.tail + 1) % BUFFER_SIZE;
            self.empty = false;
        }
    }

    /// Pop a byte from the buffer
    fn pop(&mut self) -> Option<u8> {
        if self.empty {
            None
        } else {
            let ret = self.buffer[self.head];
            self.head = (self.head + 1) % BUFFER_SIZE;
            if self.head == self.tail {
                self.empty = true;
            }
            Some(ret)
        }
    }
}

/// The UART driver
struct UartDrv {
    inner: Option<VirtIoConsoleDev<VirtIoHalImpl, VirtIoTransport>>,
    buffer: [u8; MEM_SIZE],
    #[cfg(feature = "irq")]
    irq_buffer: RxRingBuffer,
    pointer: usize,
    addr: usize,
}

/// The UART driver instance
static UART: SpinNoIrq<UartDrv> = SpinNoIrq::new(UartDrv {
    inner: None,
    buffer: [0; MEM_SIZE],
    #[cfg(feature = "irq")]
    irq_buffer: RxRingBuffer::new(),
    pointer: 0,
    addr: 0,
});

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart_drv = UART.lock();
    if uart_drv.inner.is_some() {
        if uart_drv.pointer > 0 {
            for i in 0..uart_drv.pointer {
                let c = uart_drv.buffer[i];
                let uart = uart_drv.inner.as_mut().unwrap();
                match c {
                    b'\n' => {
                        uart.putchar(b'\r');
                        uart.putchar(b'\n');
                    }
                    c => uart.putchar(c),
                }
            }
            uart_drv.pointer = 0;
            warn!("######################### The above content is printed from buffer! #########################");
        }
        let uart = uart_drv.inner.as_mut().unwrap();
        uart.putchar(c);
    } else {
        let ptr = uart_drv.pointer;
        uart_drv.buffer[ptr] = c;
        uart_drv.pointer += 1;
    }
}

/// Reads a byte from the console.
pub fn getchar() -> Option<u8> {
    let mut uart_drv = UART.lock();
    #[cfg(feature = "irq")]
    return uart_drv.irq_buffer.pop();
    #[cfg(not(feature = "irq"))]
    if let Some(ref mut uart_inner) = uart_drv.inner {
        return uart_inner.getchar();
    } else {
        None
    }
}

/// probe virtio console directly
pub fn directional_probing() {
    info!("Initiating VirtIO Console ...");
    let mut uart_drv = UART.lock();
    if let Some(dev) = probe_mmio(VIRTIO_CONSOLE_BASE, VIRTIO_CONSOLE_REG) {
        uart_drv.inner = Some(dev);
        uart_drv.addr = VIRTIO_CONSOLE_BASE;
    }
    info!("Output now redirected to VirtIO Console!");
}

/// enable virtio console interrupt
#[cfg(feature = "irq")]
pub fn enable_interrupt() {
    #[cfg(target_arch = "aarch64")]
    {
        info!("Initiating VirtIO Console interrupt ...");
        info!("IRQ ID: {}", VIRTIO_CONSOLE_IRQ_NUM);
        crate::irq::register_handler(VIRTIO_CONSOLE_IRQ_NUM, irq_handler);
        crate::irq::set_enable(VIRTIO_CONSOLE_IRQ_NUM, true);
        ack_interrupt();
        info!("Interrupt enabled!");
    }
    #[cfg(not(target_arch = "aarch64"))]
    warn!("Interrupt is not supported on this platform!");
}

/// virtio console interrupt handler
#[cfg(feature = "irq")]
pub fn irq_handler() {
    let mut uart_drv = UART.lock();
    if let Some(ref mut uart_inner) = uart_drv.inner {
        let uart = uart_inner;
        if uart.ack_interrupt().unwrap() {
            if let Some(c) = uart.getchar() {
                uart_drv.irq_buffer.push(c);
            }
        }
    }
}

/// Acknowledge the interrupt
#[cfg(feature = "irq")]
pub fn ack_interrupt() {
    info!("ack interrupt");
    let mut uart_drv = UART.lock();
    if let Some(ref mut uart_inner) = uart_drv.inner {
        let uart = uart_inner;
        uart.ack_interrupt()
            .expect("Virtio_console ack interrupt error");
    }
}

/// Check if the address is the probe address
pub fn is_probe(addr: usize) -> bool {
    let uart_drv = UART.lock();
    addr == uart_drv.addr
}

/// Probe the virtio console
fn probe_mmio(
    mmio_base: usize,
    mmio_size: usize,
) -> Option<VirtIoConsoleDev<VirtIoHalImpl, VirtIoTransport>> {
    let base_vaddr = phys_to_virt(mmio_base.into());
    if let Some((ty, transport)) =
        driver_virtio::probe_mmio_device(base_vaddr.as_mut_ptr(), mmio_size)
    {
        if ty == driver_common::DeviceType::Char {
            info!(
                "VirtIO Console found at {:#x} size {:#x}",
                mmio_base, mmio_size
            );
            return match VirtIoConsoleDev::try_new(transport) {
                Ok(dev) => Some(dev),
                Err(_e) => None,
            };
        }
    }
    None
}

/// Virtio transport type
type VirtIoTransport = driver_virtio::MmioTransport;
