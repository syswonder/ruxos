/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![no_std]
#![doc = include_str!("../README.md")]

use core::sync::atomic::{AtomicUsize, Ordering};

/// The type of an event handler.
///
/// Currently no arguments and return values are supported.
pub type Handler = fn();

/// A lock-free table of event handlers.
///
/// It internally uses an array of `AtomicUsize` to store the handlers.
pub struct HandlerTable<const N: usize> {
    handlers: [AtomicUsize; N],
}

impl<const N: usize> HandlerTable<N> {
    /// Creates a new handler table with all entries empty.
    #[allow(clippy::declare_interior_mutable_const)]
    pub const fn new() -> Self {
        const EMPTY: AtomicUsize = AtomicUsize::new(0);
        Self {
            handlers: [EMPTY; N],
        }
    }

    /// Registers a handler for the given index.
    pub fn register_handler(&self, idx: usize, handler: Handler) -> bool {
        self.handlers[idx]
            .compare_exchange(0, handler as usize, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    /// Handles the event with the given index.
    ///
    /// Returns `true` if the event is handled, `false` if no handler is
    /// registered for the given index.
    pub fn handle(&self, idx: usize) -> bool {
        let handler = self.handlers[idx].load(Ordering::Acquire);
        if handler != 0 {
            let handler: Handler = unsafe { core::mem::transmute(handler) };
            handler();
            true
        } else {
            false
        }
    }
}
