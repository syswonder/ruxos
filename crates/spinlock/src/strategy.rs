/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::marker::PhantomData;

// Re-exports extra strategies when feature enabled.
#[cfg(feature = "rand")]
pub use crate::rand_strategy::*;

#[inline(always)]
fn exp_backoff(current_limit: &mut u32, max: u32) {
    let limit = *current_limit;
    *current_limit = max.max(limit << 1);
    for _ in 0..limit {
        core::hint::spin_loop();
    }
}

/// Defines the backoff behavior of a spinlock.
pub trait Backoff {
    /// Backoff behavior when failed to acquire the lock.
    fn backoff(&mut self);
}

/// Defines the relax behavior of a spinlock.
pub trait Relax {
    /// Relax behavior when the lock seemed still held.
    fn relax(&mut self);
}

/// Defines the lock behavior when encountering contention.
/// [`Backoff::backoff`] is called when failed to acquire the lock, and
/// [`Relax::relax`]` is called when the lock seemed still held.
///
/// One can easily define a new [`Strategy`] impl that
/// combines existing backoff/relax behaviors.
pub trait Strategy {
    /// The type that defines the relax behavior.
    type Relax: Relax;

    /// The type that defines the backoff behavior.
    type Backoff: Backoff;

    /// Create a new relax state every time after failed to acquire the lock.
    fn new_relax() -> Self::Relax;

    /// Create a new backoff state every time after the locking procedure began.
    fn new_backoff() -> Self::Backoff;
}

impl<T: Relax + Backoff + Default> Strategy for T {
    type Relax = T;
    type Backoff = T;

    #[inline(always)]
    fn new_relax() -> Self::Relax {
        T::default()
    }

    #[inline(always)]
    fn new_backoff() -> Self::Backoff {
        T::default()
    }
}

/// Do nothing when backoff/relax is required.
/// It can be used as a baseline, or under rare circumstances be used as a
/// performance improvement.
///
/// Note that under most modern CPU design, not using any backoff/relax strategy
/// would normally make things slower.
#[derive(Debug, Default)]
pub struct NoOp;

/// Call [`core::hint::spin_loop`] once when backoff/relax is required.
///
/// This may improve performance by said, reducing bus traffic. The exact
/// behavior and benefits depend on the machine.
#[derive(Debug, Default)]
pub struct Once;

/// Call [`core::hint::spin_loop`] with exponentially increased time when
/// backoff/relax is required.
///
/// This would generally increase performance when the lock is highly contended.
#[derive(Debug)]
pub struct Exp<const MAX: u32>(u32);

/// Combines a [`Relax`] and a [`Backoff`] into a strategy.
#[derive(Debug, Default)]
pub struct Combine<R: Relax, B: Backoff>(PhantomData<(R, B)>);

impl Relax for NoOp {
    #[inline(always)]
    fn relax(&mut self) {}
}

impl Backoff for NoOp {
    #[inline(always)]
    fn backoff(&mut self) {}
}

impl Relax for Once {
    #[inline(always)]
    fn relax(&mut self) {
        core::hint::spin_loop();
    }
}

impl Backoff for Once {
    #[inline(always)]
    fn backoff(&mut self) {
        core::hint::spin_loop();
    }
}

impl<const N: u32> Relax for Exp<N> {
    #[inline(always)]
    fn relax(&mut self) {
        exp_backoff(&mut self.0, N);
    }
}

impl<const N: u32> Backoff for Exp<N> {
    #[inline(always)]
    fn backoff(&mut self) {
        exp_backoff(&mut self.0, N);
    }
}

impl<const N: u32> Default for Exp<N> {
    #[inline(always)]
    fn default() -> Self {
        Self(1)
    }
}

impl<R, B> Strategy for Combine<R, B>
where
    R: Relax + Default,
    B: Backoff + Default,
{
    type Relax = R;
    type Backoff = B;

    #[inline(always)]
    fn new_relax() -> Self::Relax {
        R::default()
    }

    #[inline(always)]
    fn new_backoff() -> Self::Backoff {
        B::default()
    }
}
