/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Useful synchronization primitives.

#[doc(no_inline)]
pub use core::sync::atomic;

#[cfg(feature = "alloc")]
#[doc(no_inline)]
pub use alloc::sync::{Arc, Weak};

#[cfg(feature = "multitask")]
mod mutex;

#[cfg(feature = "multitask")]
#[doc(cfg(feature = "multitask"))]
pub use self::mutex::{Mutex, MutexGuard};

#[cfg(not(feature = "multitask"))]
#[doc(cfg(not(feature = "multitask")))]
pub use spinlock::{SpinRaw as Mutex, SpinRawGuard as MutexGuard}; // never used in IRQ context
