/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::{
    ffi::c_void,
    mem::MaybeUninit,
    sync::atomic::{AtomicBool, AtomicPtr},
};
use lazy_init::LazyInit;
use spinlock::SpinNoIrq;

/// Destroy a specific key when a thread exits.
pub type DestrFunction = unsafe extern "C" fn(*mut c_void);
/// Thread-specific data set.
pub(crate) type TSD = SpinNoIrq<[*mut c_void; ruxconfig::PTHREAD_KEY_MAX]>;

/// A key for a process.
#[derive(Default)]
#[allow(dead_code)]
pub(crate) struct PthreadKey {
    in_use: AtomicBool,
    // AtomicPtr<c_void> means *mut c_void. It should be convert to DestrFunction when use.
    destr_function: AtomicPtr<c_void>,
}

impl PthreadKey {
    /// Create a new key.
    pub fn new() -> Self {
        Self {
            in_use: AtomicBool::new(false),
            destr_function: AtomicPtr::new(core::ptr::null_mut()),
        }
    }
}

/// A set of keys for a process.
pub(crate) struct PthreadKeys {
    keys: [PthreadKey; ruxconfig::PTHREAD_KEY_MAX],
}

impl PthreadKeys {
    /// Create a new key set.
    pub fn new() -> Self {
        let mut arr: [MaybeUninit<PthreadKey>; ruxconfig::PTHREAD_KEY_MAX] =
            unsafe { MaybeUninit::uninit().assume_init() };
        for a in arr.iter_mut() {
            *a = MaybeUninit::new(PthreadKey::new());
        }
        Self {
            keys: unsafe {
                core::mem::transmute::<_, [PthreadKey; ruxconfig::PTHREAD_KEY_MAX]>(arr)
            },
        }
    }

    /// Allocate a key
    pub fn alloc(&self, destr_function: Option<DestrFunction>) -> Option<usize> {
        for (i, key) in self.keys.iter().enumerate() {
            if !key.in_use.load(core::sync::atomic::Ordering::Relaxed) {
                key.in_use
                    .store(true, core::sync::atomic::Ordering::Relaxed);
                if let Some(destr_function) = destr_function {
                    key.destr_function.store(
                        destr_function as *mut c_void,
                        core::sync::atomic::Ordering::Relaxed,
                    );
                } else {
                    key.destr_function
                        .store(core::ptr::null_mut(), core::sync::atomic::Ordering::Relaxed);
                }
                return Some(i);
            }
        }
        None
    }

    /// Free a key
    pub fn free(&self, key: usize) -> Option<()> {
        if key < self.keys.len() {
            self.keys[key]
                .in_use
                .store(false, core::sync::atomic::Ordering::Relaxed);
            Some(())
        } else {
            None
        }
    }

    /// Get all keys used
    pub fn destr_used_keys(&self, tsd: &TSD) {
        for (i, key) in self.keys.iter().enumerate() {
            if key.in_use.load(core::sync::atomic::Ordering::Relaxed) {
                let destr_function = key
                    .destr_function
                    .load(core::sync::atomic::Ordering::Relaxed);
                if !destr_function.is_null() {
                    unsafe {
                        let destr_function =
                            core::mem::transmute::<*mut c_void, DestrFunction>(destr_function);
                        destr_function(tsd.lock()[i]);
                    }
                }
            }
        }
    }
}

/// Instance of a thread-shared key set.
pub(crate) static mut KEYS: LazyInit<SpinNoIrq<PthreadKeys>> = LazyInit::new();

/// Initialize the thread-shared key set.
pub(crate) fn init() {
    unsafe {
        KEYS.init_by(SpinNoIrq::new(PthreadKeys::new()));
    }
}
