//! Syscall dispatch crate
//!
//! Dispatch musl syscall instruction to Ruxos posix-api
//!
//! Only support AARCH64 right now

#![cfg_attr(all(not(test), not(doc)), no_std)]

#[macro_use]
extern crate axlog;

#[cfg(feature = "alloc")]
extern crate alloc;

mod syscall;
mod trap;
