/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! A naïve spinning mutex.
//!
//! Waiting threads hammer an atomic variable until it becomes available. Best-case latency is low, but worst-case
//! latency is theoretically infinite.
//!
//! Based on [`spin::Mutex`](https://docs.rs/spin/latest/src/spin/mutex/spin.rs.html).

use core::cell::UnsafeCell;
use core::fmt;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

#[cfg(feature = "smp")]
use core::sync::atomic::{AtomicBool, Ordering};

use kernel_guard::BaseGuard;

use crate::{strategy, Strategy};

/// The default strategy used in spinlocks.
pub type DefaultStrategy = strategy::Once;

/// A [spin lock](https://en.m.wikipedia.org/wiki/Spinlock) providing mutually
/// exclusive access to data.
///
/// This is a base struct, the specific behavior depends on the generic
/// parameter `G` that implements [`BaseGuard`], such as whether to disable
/// local IRQs or kernel preemption before acquiring the lock. The parameter `S`
/// that implements [`Strategy`] defines the behavior when encountering contention.
///
/// For single-core environment (without the "smp" feature), we remove the lock
/// state, CPU can always get the lock if we follow the proper guard in use.
pub struct BaseSpinLock<DG: BaseGuard, T: ?Sized, S: Strategy = DefaultStrategy> {
    _phantom: PhantomData<(DG, S)>,
    #[cfg(feature = "smp")]
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

/// A guard that provides mutable data access.
///
/// When the guard falls out of scope it will release the lock.
pub struct BaseSpinLockGuard<'a, G: BaseGuard, T: ?Sized + 'a> {
    _phantom: &'a PhantomData<G>,
    irq_state: G::State,
    data: *mut T,
    #[cfg(feature = "smp")]
    lock: &'a AtomicBool,
}

// Same unsafe impls as `std::sync::Mutex`
unsafe impl<G: BaseGuard, T: ?Sized + Send, B: Strategy> Sync for BaseSpinLock<G, T, B> {}
unsafe impl<G: BaseGuard, T: ?Sized + Send, B: Strategy> Send for BaseSpinLock<G, T, B> {}

impl<G: BaseGuard, T, S: Strategy> BaseSpinLock<G, T, S> {
    /// Creates a new [`BaseSpinLock`] wrapping the supplied data.
    #[inline(always)]
    pub const fn new(data: T) -> Self {
        Self {
            _phantom: PhantomData,
            data: UnsafeCell::new(data),
            #[cfg(feature = "smp")]
            lock: AtomicBool::new(false),
        }
    }

    /// Consumes this [`BaseSpinLock`] and unwraps the underlying data.
    #[inline(always)]
    pub fn into_inner(self) -> T {
        // We know statically that there are no outstanding references to
        // `self` so there's no need to lock.
        let BaseSpinLock { data, .. } = self;
        data.into_inner()
    }
}

impl<G: BaseGuard, T: ?Sized, S: Strategy> BaseSpinLock<G, T, S> {
    /// Locks the [`BaseSpinLock`] using the given guard type and backoff strategy,
    /// and returns a guard that permits access to the inner data.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    pub fn lock_as<GT: BaseGuard, ST: Strategy>(&self) -> BaseSpinLockGuard<GT, T> {
        let irq_state = GT::acquire();

        #[cfg(feature = "smp")]
        {
            use crate::strategy::{Backoff, Relax};

            let mut backoff = <ST as Strategy>::new_backoff();
            // Can fail to lock even if the spinlock is not locked. May be more efficient than `try_lock`
            // when called in a loop.
            while self
                .lock
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_err()
            {
                backoff.backoff();
                let mut relax = <ST as Strategy>::new_relax();

                // Wait until the lock looks unlocked before retrying
                while self.is_locked() {
                    relax.relax();
                }
            }
        }
        BaseSpinLockGuard {
            _phantom: &PhantomData,
            irq_state,
            data: unsafe { &mut *self.data.get() },
            #[cfg(feature = "smp")]
            lock: &self.lock,
        }
    }

    /// Locks the [`BaseSpinLock`] using the "default" strategy specified by lock type,
    /// and returns a guard that permits access to the inner data.
    ///
    /// The returned value may be dereferenced for data access
    /// and the lock will be dropped when the guard falls out of scope.
    #[inline(always)]
    pub fn lock(&self) -> BaseSpinLockGuard<G, T> {
        self.lock_as::<G, S>()
    }

    /// Returns `true` if the lock is currently held.
    ///
    /// # Safety
    ///
    /// This function provides no synchronization guarantees and so its result should be considered 'out of date'
    /// the instant it is called. Do not use it for synchronization purposes. However, it may be useful as a heuristic.
    #[inline(always)]
    pub fn is_locked(&self) -> bool {
        cfg_if::cfg_if! {
            if #[cfg(feature = "smp")] {
                self.lock.load(Ordering::Relaxed)
            } else {
                false
            }
        }
    }

    /// Try to lock this [`BaseSpinLock`], returning a lock guard if successful.
    #[inline(always)]
    pub fn try_lock(&self) -> Option<BaseSpinLockGuard<G, T>> {
        let irq_state = G::acquire();

        cfg_if::cfg_if! {
            if #[cfg(feature = "smp")] {
                // The reason for using a strong compare_exchange is explained here:
                // https://github.com/Amanieu/parking_lot/pull/207#issuecomment-575869107
                let is_unlocked = self
                .lock
                .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok();
            } else {
                let is_unlocked = true;
            }
        }

        if is_unlocked {
            Some(BaseSpinLockGuard {
                _phantom: &PhantomData,
                irq_state,
                data: unsafe { &mut *self.data.get() },
                #[cfg(feature = "smp")]
                lock: &self.lock,
            })
        } else {
            None
        }
    }

    /// Force unlock this [`BaseSpinLock`].
    ///
    /// # Safety
    ///
    /// This is *extremely* unsafe if the lock is not held by the current
    /// thread. However, this can be useful in some instances for exposing the
    /// lock to FFI that doesn't know how to deal with RAII.
    #[inline(always)]
    pub unsafe fn force_unlock(&self) {
        #[cfg(feature = "smp")]
        self.lock.store(false, Ordering::Release);
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the [`BaseSpinLock`] mutably, and a mutable reference is guaranteed to be exclusive in
    /// Rust, no actual locking needs to take place -- the mutable borrow statically guarantees no locks exist. As
    /// such, this is a 'zero-cost' operation.
    #[inline(always)]
    pub fn get_mut(&mut self) -> &mut T {
        // We know statically that there are no other references to `self`, so
        // there's no need to lock the inner mutex.
        unsafe { &mut *self.data.get() }
    }
}

impl<G: BaseGuard, T: ?Sized + Default, S: Strategy> Default for BaseSpinLock<G, T, S> {
    #[inline(always)]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

impl<G: BaseGuard, T: ?Sized + fmt::Debug, S: Strategy> fmt::Debug for BaseSpinLock<G, T, S> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.try_lock() {
            Some(guard) => write!(f, "SpinLock {{ data: ")
                .and_then(|()| (*guard).fmt(f))
                .and_then(|()| write!(f, "}}")),
            None => write!(f, "SpinLock {{ <locked> }}"),
        }
    }
}

impl<'a, G: BaseGuard, T: ?Sized> Deref for BaseSpinLockGuard<'a, G, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        // We know statically that only we are referencing data
        unsafe { &*self.data }
    }
}

impl<'a, G: BaseGuard, T: ?Sized> DerefMut for BaseSpinLockGuard<'a, G, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // We know statically that only we are referencing data
        unsafe { &mut *self.data }
    }
}

impl<'a, G: BaseGuard, T: ?Sized + fmt::Debug> fmt::Debug for BaseSpinLockGuard<'a, G, T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, G: BaseGuard, T: ?Sized> Drop for BaseSpinLockGuard<'a, G, T> {
    /// The dropping of the [`BaseSpinLockGuard`] will release the lock it was
    /// created from.
    #[inline(always)]
    fn drop(&mut self) {
        #[cfg(feature = "smp")]
        self.lock.store(false, Ordering::Release);
        G::release(self.irq_state);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::mpsc::channel;
    use std::sync::Arc;
    use std::thread;

    type SpinMutex<T> = crate::SpinRaw<T>;

    #[derive(Eq, PartialEq, Debug)]
    struct NonCopy(i32);

    #[test]
    fn smoke() {
        let m = SpinMutex::<_>::new(());
        drop(m.lock());
        drop(m.lock());
    }

    #[test]
    #[cfg(feature = "smp")]
    fn lots_and_lots() {
        static M: SpinMutex<()> = SpinMutex::<_>::new(());
        static mut CNT: u32 = 0;
        const J: u32 = 1000;
        const K: u32 = 3;

        fn inc() {
            for _ in 0..J {
                unsafe {
                    let _g = M.lock();
                    CNT += 1;
                }
            }
        }

        let (tx, rx) = channel();
        let mut ts = Vec::new();
        for _ in 0..K {
            let tx2 = tx.clone();
            ts.push(thread::spawn(move || {
                inc();
                tx2.send(()).unwrap();
            }));
            let tx2 = tx.clone();
            ts.push(thread::spawn(move || {
                inc();
                tx2.send(()).unwrap();
            }));
        }

        drop(tx);
        for _ in 0..2 * K {
            rx.recv().unwrap();
        }
        assert_eq!(unsafe { CNT }, J * K * 2);

        for t in ts {
            t.join().unwrap();
        }
    }

    #[test]
    #[cfg(feature = "smp")]
    fn try_lock() {
        let mutex = SpinMutex::<_>::new(42);

        // First lock succeeds
        let a = mutex.try_lock();
        assert_eq!(a.as_ref().map(|r| **r), Some(42));

        // Additional lock fails
        let b = mutex.try_lock();
        assert!(b.is_none());

        // After dropping lock, it succeeds again
        ::core::mem::drop(a);
        let c = mutex.try_lock();
        assert_eq!(c.as_ref().map(|r| **r), Some(42));
    }

    #[test]
    fn test_into_inner() {
        let m = SpinMutex::<_>::new(NonCopy(10));
        assert_eq!(m.into_inner(), NonCopy(10));
    }

    #[test]
    fn test_into_inner_drop() {
        struct Foo(Arc<AtomicUsize>);
        impl Drop for Foo {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let num_drops = Arc::new(AtomicUsize::new(0));
        let m = SpinMutex::<_>::new(Foo(num_drops.clone()));
        assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        {
            let _inner = m.into_inner();
            assert_eq!(num_drops.load(Ordering::SeqCst), 0);
        }
        assert_eq!(num_drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_mutex_arc_nested() {
        // Tests nested mutexes and access
        // to underlying data.
        let arc = Arc::new(SpinMutex::<_>::new(1));
        let arc2 = Arc::new(SpinMutex::<_>::new(arc));
        let (tx, rx) = channel();
        let t = thread::spawn(move || {
            let lock = arc2.lock();
            let lock2 = lock.lock();
            assert_eq!(*lock2, 1);
            tx.send(()).unwrap();
        });
        rx.recv().unwrap();
        t.join().unwrap();
    }

    #[test]
    fn test_mutex_arc_access_in_unwind() {
        let arc = Arc::new(SpinMutex::<_>::new(1));
        let arc2 = arc.clone();
        let _ = thread::spawn(move || {
            struct Unwinder {
                i: Arc<SpinMutex<i32>>,
            }
            impl Drop for Unwinder {
                fn drop(&mut self) {
                    *self.i.lock() += 1;
                }
            }
            let _u = Unwinder { i: arc2 };
            panic!();
        })
        .join();
        let lock = arc.lock();
        assert_eq!(*lock, 2);
    }

    #[test]
    fn test_mutex_unsized() {
        let mutex: &SpinMutex<[i32]> = &SpinMutex::<_>::new([1, 2, 3]);
        {
            let b = &mut *mutex.lock();
            b[0] = 4;
            b[2] = 5;
        }
        let comp: &[i32] = &[4, 2, 5];
        assert_eq!(&*mutex.lock(), comp);
    }

    #[test]
    fn test_mutex_force_lock() {
        let lock = SpinMutex::<_>::new(());
        ::std::mem::forget(lock.lock());
        unsafe {
            lock.force_unlock();
        }
        assert!(lock.try_lock().is_some());
    }
}
