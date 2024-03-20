/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! PL011 UART.
use arm_pl011::pl011::Pl011Uart;
use cfg_if::cfg_if;
use memory_addr::PhysAddr;
use spinlock::SpinNoIrq;

use crate::mem::phys_to_virt;

const UART_BASE: PhysAddr = PhysAddr::from(ruxconfig::UART_PADDR);
#[cfg(feature = "irq")]
const BUFFER_SIZE: usize = 128;

#[cfg(feature = "irq")]
struct RxRingBuffer {
    buffer: [u8; BUFFER_SIZE],
    head: usize,
    tail: usize,
}

#[cfg(feature = "irq")]
impl RxRingBuffer {
    const fn new() -> Self {
        RxRingBuffer {
            buffer: [0_u8; BUFFER_SIZE],
            head: 0_usize,
            tail: 0_usize,
        }
    }

    fn push(&mut self, n: u8) {
        if self.tail != self.head {
            self.buffer[self.tail] = n;
            self.tail = (self.tail + 1) % BUFFER_SIZE;
        }
    }

    fn pop(&mut self) -> Option<u8> {
        if self.head == self.tail {
            None
        } else {
            let ret = self.buffer[self.head];
            self.head += (self.head + 1) % BUFFER_SIZE;
            Some(ret)
        }
    }
}

struct UartDrv {
    inner: SpinNoIrq<Pl011Uart>,
    #[cfg(feature = "irq")]
    buffer: SpinNoIrq<RxRingBuffer>,
}

static UART: UartDrv = UartDrv {
    inner: SpinNoIrq::new(Pl011Uart::new(phys_to_virt(UART_BASE).as_mut_ptr())),
    #[cfg(feature = "irq")]
    buffer: SpinNoIrq::new(RxRingBuffer::new()),
};

/// Writes a byte to the console.
pub fn putchar(c: u8) {
    let mut uart = UART.inner.lock();
    match c {
        b'\n' => {
            uart.putchar(b'\r');
            uart.putchar(b'\n');
        }
        c => uart.putchar(c),
    }
}

/// Reads a byte from the console, or returns [`None`] if no input is available.
pub fn getchar() -> Option<u8> {
    cfg_if! {
        if #[cfg(feature = "irq")] {
            UART.buffer.lock().pop()
        }else{
            UART.inner.lock().getchar()
        }
    }
}

/// Initialize the UART
pub fn init_early() {
    UART.inner.lock().init();
}

/// Set UART IRQ Enable
pub fn init() {
    #[cfg(feature = "irq")]
    {
        crate::irq::register_handler(crate::platform::irq::UART_IRQ_NUM, handle);
        crate::irq::set_enable(crate::platform::irq::UART_IRQ_NUM, true);
    }
}

/// UART IRQ Handler
#[cfg(feature = "irq")]
pub fn handle() {
    let is_receive_interrupt = UART.inner.lock().is_receive_interrupt();
    if is_receive_interrupt {
        UART.inner.lock().ack_interrupts();
        while let Some(c) = UART.inner.lock().getchar() {
            UART.buffer.lock().push(c);
        }
    }
}
