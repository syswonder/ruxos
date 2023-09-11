//! Syscall dispatch crate
//!
//! Dispatch musl syscall instruction to Rukos posix-api
//!
//! Only support AARCH64 right now

#![cfg_attr(all(not(test), not(doc)), no_std)]

#[macro_use]
extern crate axlog;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "irq")]
extern crate axruntime;

mod syscall;

#[cfg(feature = "net")]
mod net;
mod trap;

use core::ffi::c_int;

pub fn e(ret: c_int) -> c_int {
    if ret < 0 {
        -1
    } else {
        ret as _
    }
}
