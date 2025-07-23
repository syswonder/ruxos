/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

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

mod trap;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "aarch64")]{
        mod aarch64;
        use aarch64::{syscall, syscall_id};
    } else if #[cfg(target_arch = "x86_64")]{
        mod x86_64;
        #[cfg(feature = "musl")]
        use x86_64::{syscall, syscall_id};
    } else if #[cfg(target_arch = "riscv64")]{
        mod riscv64;
        use riscv64::{syscall, syscall_id};
    } else {
        mod dummy;
        use dummy::{syscall, syscall_id};
    }
}
