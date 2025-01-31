/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos] hardware abstraction layer, provides unified APIs for
//! platform-specific operations.
//!
//! It does the bootstrapping and initialization process for the specified
//! platform, and provides useful operations on the hardware.
//!
//! Currently supported platforms (specify by cargo features):
//!
//! - `x86-pc`: Standard PC with x86_64 ISA.
//! - `riscv64-qemu-virt`: QEMU virt machine with RISC-V ISA.
//! - `aarch64-qemu-virt`: QEMU virt machine with AArch64 ISA.
//! - `aarch64-raspi`: Raspberry Pi with AArch64 ISA.
//! - `dummy`: If none of the above platform is selected, the dummy platform
//!    will be used. In this platform, most of the operations are no-op or
//!    `unimplemented!()`. This platform is mainly used for [cargo test].
//!
//! # Cargo Features
//!
//! - `smp`: Enable SMP (symmetric multiprocessing) support.
//! - `fp_simd`: Enable floating-point and SIMD support.
//! - `paging`: Enable page table manipulation.
//! - `irq`: Enable interrupt handling support.
//!
//! [Ruxos]: https://github.com/syswonder/ruxos
//! [cargo test]: https://doc.rust-lang.org/cargo/guide/tests.html

#![no_std]
#![feature(asm_const)]
#![feature(naked_functions)]
#![feature(const_option)]
#![feature(doc_auto_cfg)]

#[allow(unused_imports)]
#[macro_use]
extern crate log;

pub mod arch;
pub mod cpu;
pub mod mem;
mod platform;
pub mod time;
pub mod trap;
pub mod virtio;

#[cfg(feature = "tls")]
pub mod tls;

#[cfg(feature = "irq")]
pub mod irq;

#[cfg(feature = "paging")]
pub mod paging;

/// Console input and output.
pub mod console {
    pub use super::platform::console::*;

    /// Write a slice of bytes to the console.
    pub fn write_bytes(bytes: &[u8]) {
        for c in bytes {
            putchar(*c);
        }
    }
}

/// Miscellaneous operation, e.g. terminate the system.
pub mod misc {
    pub use super::platform::misc::*;
}

/// Multi-core operations.
#[cfg(feature = "smp")]
pub mod mp {
    pub use super::platform::mp::*;
}

pub use self::platform::platform_init;

#[cfg(feature = "smp")]
pub use self::platform::platform_init_secondary;

/// A cmdline buf for x86_64
///
/// The Multiboot information structure may be placed anywhere in memory by the boot loader,
/// so we should save cmdline in a buf before this memory is set free
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub static mut COMLINE_BUF: [u8; 256] = [0; 256];

#[allow(unused)]
/// read a tty device specified by its name.
pub fn tty_read(buf: &mut [u8], dev_name: &str) -> usize {
    #[cfg(not(feature = "tty"))]
    {
        let mut read_len = 0;
        while read_len < buf.len() {
            if let Some(c) = console::getchar().map(|c| if c == b'\r' { b'\n' } else { c }) {
                buf[read_len] = c;
                read_len += 1;
            } else {
                break;
            }
        }
        read_len
    }

    #[cfg(feature = "tty")]
    {
        tty::tty_read(buf, dev_name)
    }
}

#[cfg(feature = "alloc")]
extern crate alloc;

/// get all tty devices' names.
#[cfg(feature = "alloc")]
pub fn get_all_device_names() -> alloc::vec::Vec<alloc::string::String> {
    #[cfg(feature = "tty")]
    {
        tty::get_all_device_names()
    }
    #[cfg(not(feature = "tty"))]
    {
        alloc::vec![alloc::string::String::from("notty")]
    }
}

/// write a tty device specified by its name.
pub fn tty_write(buf: &[u8], _dev_name: &str) -> usize {
    #[cfg(feature = "tty")]
    {
        tty::tty_write(buf, _dev_name)
    }
    #[cfg(not(feature = "tty"))]
    {
        console::write_bytes(buf);
        return buf.len();
    }
}
