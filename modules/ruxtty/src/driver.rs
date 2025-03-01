/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{sync::Arc, vec::Vec};

use crate::tty::Tty;
use spinlock::SpinNoIrq;

/// TTY device controller managing registered TTY instances
pub struct TtyDriver {
    /// When registering a tty device(e.g. ttyS), it will be put in tty device list `ttys`
    ttys: SpinNoIrq<Vec<Arc<Tty>>>,
}

impl Default for TtyDriver {
    fn default() -> Self {
        Self {
            ttys: SpinNoIrq::new(Vec::new()),
        }
    }
}

impl TtyDriver {
    /// Creates a new TTY driver with empty device list
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ttys: SpinNoIrq::new(Vec::new()),
        })
    }

    /// Registers a TTY device to the driver's management list
    pub fn add_tty(&self, tty: Arc<Tty>) {
        self.ttys.lock().push(tty)
    }

    /// Broadcasts input bytes to all registered TTY devices
    ///
    /// Sequentially sends each byte to every managed TTY
    pub fn push_slice(&self, slice: &[u8]) {
        for tty in &*self.ttys.lock() {
            for ch in slice {
                tty.push_char(*ch);
            }
        }
    }

    /// Broadcasts input byte to all registered TTY devices
    pub fn push_char(&self, ch: u8) {
        for tty in &*self.ttys.lock() {
            tty.push_char(ch);
        }
    }
}
