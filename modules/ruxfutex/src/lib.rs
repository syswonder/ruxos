/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos] futex implementation, provides a set of functions for implementing user-space synchronization primitives.
//!
//! See [futex manpage] for more details. This implementation separates different futex operations into different functions,
//! corresponding their `futex_op`s.
//!
//! [futex manpage]: https://man7.org/linux/man-pages/man2/futex.2.html
//! [Ruxos]: https://github.com/syswonder/ruxos
//! [cargo test]: https://doc.rust-lang.org/cargo/guide/tests.html

#![no_std]
#![feature(doc_auto_cfg)]

extern crate alloc;
extern crate log;

mod api;
mod types;

pub use api::{
    futex_wait, futex_wait_bitset, futex_wake, futex_wake_bitset, FUTEX_BITSET_MATCH_ANY,
};

use types::FutexVec;

use core::ops::Deref;

use lazy_static::lazy_static;

// Use the same count as linux kernel to keep the same performance
const BUCKET_COUNT: usize = ((1 << 8) * (ruxconfig::SMP)).next_power_of_two();
const BUCKET_MASK: usize = BUCKET_COUNT - 1;

lazy_static! {
    static ref FUTEX_BUCKETS: FutexVec = FutexVec::new(BUCKET_COUNT);
}

/// Inits the futex module.
pub fn init_futex() {
    let _ = FUTEX_BUCKETS.deref();
}
