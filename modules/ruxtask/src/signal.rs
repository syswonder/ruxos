/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Signal module for RuxOS.

use crate::current;
#[cfg(feature = "irq")]
use core::sync::atomic::AtomicI64;
use core::{
    ffi::{c_int, c_uint, c_ulong},
    time::Duration,
};

use ruxhal::trap::TrapHandler;

/// sigaction in kernel
#[allow(non_camel_case_types)]
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct rx_sigaction {
    /// signal handler
    pub sa_handler: Option<unsafe extern "C" fn(c_int)>,
    /// signal flags
    pub sa_flags: c_ulong,
    /// signal restorer
    pub sa_restorer: Option<unsafe extern "C" fn()>,
    /// signal mask
    pub sa_mask: [c_uint; 2usize],
}

impl rx_sigaction {
    const fn new() -> Self {
        rx_sigaction {
            sa_handler: Some(default_handler),
            sa_flags: 0,
            sa_restorer: None,
            sa_mask: [0, 0],
        }
    }
}

/// Signal struct
pub struct Signal {
    #[cfg(feature = "irq")]
    signal: AtomicI64,
    sigaction: [rx_sigaction; 32],
    timer_value: [Duration; 3],
    timer_interval: [Duration; 3],
}

unsafe extern "C" fn default_handler(signum: c_int) {
    panic!("default_handler, signum: {}", signum);
}

#[cfg(feature = "signal")]
struct SignalHandler;

#[cfg(feature = "signal")]
#[crate_interface::impl_interface]
impl TrapHandler for SignalHandler {
    fn handle_signal() {
        let signal = Signal::signal(-1, true).unwrap();
        for signum in 0..32 {
            if signal & (1 << signum) != 0
            /* TODO: && support mask */
            {
                Signal::signal(signum as i8, false);
                Signal::sigaction(signum as u8, None, None);
                Signal::signal_handle(signum as u8);
            }
        }
    }
}

impl Default for Signal {
    fn default() -> Self {
        Self::new()
    }
}

impl Signal {
    ///crate new Signal struct
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "irq")]
            signal: AtomicI64::new(0),
            sigaction: [rx_sigaction::new(); 32],
            // Default::default() is not const
            timer_value: [Duration::from_nanos(0); 3],
            timer_interval: [Duration::from_nanos(0); 3],
        }
    }
    /// Set signal
    /// signum: signal number, if signum < 0, just return current signal
    /// on: true: enable signal, false: disable signal
    #[cfg(feature = "irq")]
    pub fn signal(signum: i8, on: bool) -> Option<u32> {
        use core::sync::atomic::Ordering;
        if signum >= 32 {
            return None;
        }
        let binding = current();
        let current_signal_if = binding.signal_if.lock();
        let mut old = current_signal_if.signal.load(Ordering::Acquire);
        if signum >= 0 {
            loop {
                let new = if on {
                    old | (1 << signum)
                } else {
                    old & !(1 << signum)
                };

                match {
                    current_signal_if.signal.compare_exchange_weak(
                        old,
                        new,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                } {
                    Ok(_) => break,
                    Err(x) => old = x,
                }
            }
        }
        Some(old.try_into().unwrap())
    }
    /// Set signal action
    /// signum: signal number
    /// sigaction: signal action, if sigaction == None, call the handler
    pub fn sigaction(
        signum: u8,
        sigaction: Option<*const rx_sigaction>,
        oldact: Option<*mut rx_sigaction>,
    ) {
        let binding = current();
        let mut current_signal_if = binding.signal_if.lock();
        if signum >= current_signal_if.sigaction.len() as u8 {
            return;
        }
        if let Some(oldact) = oldact {
            unsafe {
                *oldact = current_signal_if.sigaction[signum as usize];
            }
        }

        if let Some(s) = sigaction {
            unsafe {
                current_signal_if.sigaction[signum as usize] = *s;
            }
        }
    }
    /// Handle signal for the current process
    /// signum: Signal number to handle
    /// When the registered sa_handler for the specified signal is None, logs an error message
    pub fn signal_handle(signum: u8) {
        let binding = current();
        let current_signal_if = binding.signal_if.lock();
        if let Some(handler) = current_signal_if.sigaction[signum as usize].sa_handler {
            unsafe {
                handler(signum as c_int);
            }
        } else {
            log::error!("no sigaction !");
        }
    }
    /// Set timer
    /// which: timer type
    /// new_value: new timer value
    /// old_value: old timer value
    pub fn timer_deadline(which: usize, new_deadline: Option<u64>) -> Option<u64> {
        let binding = current();
        let mut current_signal_if = binding.signal_if.lock();
        if which >= current_signal_if.timer_value.len() {
            return None;
        }
        let old = current_signal_if.timer_value[which];
        if let Some(s) = new_deadline {
            current_signal_if.timer_value[which] = Duration::from_nanos(s);
        }
        Some(old.as_nanos() as u64)
    }
    /// Set timer interval
    /// which: timer type
    /// new_interval: new timer interval
    /// old_interval: old timer interval
    pub fn timer_interval(which: usize, new_interval: Option<u64>) -> Option<u64> {
        let binding = current();
        let mut current_signal_if = binding.signal_if.lock();
        if which >= current_signal_if.timer_interval.len() {
            return None;
        }
        let old = current_signal_if.timer_interval[which];
        if let Some(s) = new_interval {
            current_signal_if.timer_interval[which] = Duration::from_nanos(s);
        }
        Some(old.as_nanos() as u64)
    }
}
