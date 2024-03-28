//! Syscall dispatch crate
//!
//! Dispatch musl syscall instruction to Ruxos posix-api
//!
//! Only support AARCH64 right now

#![feature(asm_const)]
#![feature(naked_functions)]
#![cfg_attr(all(not(test), not(doc)), no_std)]

#[macro_use]
extern crate axlog;

#[cfg(feature = "alloc")]
extern crate alloc;

mod trap;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")]{
        mod aarch64;
        use aarch64::{syscall, syscall_id};
    } else if #[cfg(target_arch = "x86_64")]{
        mod x86_64;
        use x86_64::{syscall, syscall_id};
    } else if #[cfg(target_arch = "riscv64")]{
        mod riscv64;
        use riscv64::{syscall, syscall_id};
    } else {
        mod dummy;
        use dummy::{syscall, syscall_id};
    }
}
