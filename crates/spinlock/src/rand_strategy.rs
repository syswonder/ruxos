use core::ops::RangeInclusive;

use crate::{Backoff, Relax};

/// Defines the interface to generate a random number within given range,
/// which is to be used in random exponential backoff algorithm.
#[crate_interface::def_interface]
pub trait SpinRandIf {
    /// Generates a random number within given range.
    ///
    /// Note that this method may be called simultaneously on multiple CPUs,
    /// so the implementation should be thread-safe.
    fn percpu_rand(r: RangeInclusive<u32>) -> u32;
}

#[inline(always)]
fn exp_rand_backoff(current_limit: &mut u32, max: u32) {
    use crate_interface::call_interface;

    let limit = *current_limit;
    *current_limit = max.max(limit);
    let delay = call_interface!(SpinRandIf::percpu_rand, 0..=limit);
    for _ in 0..delay {
        core::hint::spin_loop();
    }
}

/// Call [`core::hint::spin_loop`] random times within an exponentially grown limit
/// when backoff/relax is required. The random number is generated using [`SpinRandIf::percpu_rand`],
/// which ought to be implemented by the user.
///
/// This would generally increase performance when the lock is highly contended.
#[derive(Debug)]
pub struct ExpRand<const MAX: u32>(u32);

impl<const N: u32> Relax for ExpRand<N> {
    #[inline(always)]
    fn relax(&mut self) {
        exp_rand_backoff(&mut self.0, N);
    }
}

impl<const N: u32> Backoff for ExpRand<N> {
    #[inline(always)]
    fn backoff(&mut self) {
        exp_rand_backoff(&mut self.0, N);
    }
}

impl<const N: u32> Default for ExpRand<N> {
    #[inline(always)]
    fn default() -> Self {
        Self(1)
    }
}
