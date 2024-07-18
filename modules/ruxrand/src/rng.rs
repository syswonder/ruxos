/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use lazy_init::LazyInit;
use percpu::def_percpu;
use rand::{distributions::Standard, prelude::*};
use rand_xoshiro::Xoshiro256StarStar;

#[allow(clippy::unusual_byte_groupings)]
const RNG_SEED: u64 = 0xBAD_C0FFEE_0DD_F00D;

type PercpuRngType = Xoshiro256StarStar;

#[def_percpu]
pub(crate) static PERCPU_RNG: LazyInit<PercpuRngType> = LazyInit::new();

/// Initializes the per-CPU random number generator (RNG).
///
/// This function seeds the RNG with a hard-coded seed value and then performs a
/// series of long jumps so that the random number sequence on each CPU is guaranteed to
/// not overlap with each other.
///
/// A single [`PercpuRngType::long_jump`] skips 2^192 random numbers, providing sufficient
/// space for randomness on each CPU.
pub(crate) fn init(cpuid: usize) {
    PERCPU_RNG.with_current(|percpu_ref| {
        let mut rng = PercpuRngType::seed_from_u64(RNG_SEED);
        for _ in 0..cpuid {
            rng.long_jump();
        }

        percpu_ref.init_by(rng);
    });
}

// Rationale for using a raw pointer in `PercpuRng`:
//
// Just like the case in `rand::thread_rng`, there will only
// ever be one mutable reference generated from the mutable pointer, because
// we only have such a reference inside `next_u32`, `next_u64`, etc. Within a
// single processor (which is the definition of `PercpuRng`), there will only ever
// be one of these methods active at a time.
//
// A possible scenario where there could be multiple mutable references is if
// `PercpuRng` is used inside `next_u32` and co. But the implementation is
// completely under our control. We just have to ensure none of them use
// `PercpuRng` internally, which is nonsensical anyway. We should also never run
// `PercpuRng` in destructors of its implementation, which is also nonsensical.
//
// Another possible scenario is that an interrupt happens at the middle of `next_u32`
// or so, and the interrupt handler uses `PercpuRng`. This is indeed a violation of the
// Rust aliasing model, but can hardly lead to any true hazard I think. It can be easily
// fixed by requiring no IRQ using `kernel_guard` when implementing the functions provided
// by `RngCore`.

/// A RNG wrapper that's local to the calling CPU. The actual RNG type is
/// `PercpuRngType`, which is currently [`Xoshiro256StarStar`].
///
/// This type is ! [`Send`] and ! [`Sync`], preventing potential misuse under
/// SMP environments. Construct this type using [`Default::default`], or just
/// a call to [`percpu_rng`].
#[derive(Clone, Debug)]
pub struct PercpuRng {
    rng: *mut PercpuRngType,
}

impl PercpuRng {
    fn get_rng(&mut self) -> &mut PercpuRngType {
        unsafe { &mut *self.rng }
    }
}

impl Default for PercpuRng {
    fn default() -> Self {
        percpu_rng()
    }
}

impl RngCore for PercpuRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.get_rng().next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.get_rng().next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.get_rng().fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.get_rng().try_fill_bytes(dest)
    }
}

impl CryptoRng for PercpuRng {}

/// Retrieves the per-CPU RNG [`PercpuRng`] that points to the CPU-local RNG states.
///
/// The RNGs were initialized by the same seed, but jumped to different locations in
/// the pseudo-random sequence, effectively making the RNG independently and identically
/// distributed on all CPUs.
pub fn percpu_rng() -> PercpuRng {
    // It is unsafe to return mutable pointer to a global data structure
    // without preemption disabled, but the baddest thing that can happen whatsoever
    // here is the rng being put into some "random" state, which I think is not fatal.
    let rng = unsafe { PERCPU_RNG.current_ref_mut_raw().get_mut_unchecked() as *mut _ };
    PercpuRng { rng }
}

/// Generates a random value using the per-CPU random number generator.
///
/// This is simply a shortcut for `percpu_rng().gen()`. See [`percpu_rng`] for
/// documentation of the entropy source and [`Standard`] for documentation of
/// distributions and type-specific generation.
///
/// # Provided implementations
///
/// The following types have provided implementations that
/// generate values with the following ranges and distributions:
///
/// * Integers (`i32`, `u32`, `isize`, `usize`, etc.): Uniformly distributed
///   over all values of the type.
/// * `char`: Uniformly distributed over all Unicode scalar values, i.e. all
///   code points in the range `0...0x10_FFFF`, except for the range
///   `0xD800...0xDFFF` (the surrogate code points). This includes
///   unassigned/reserved code points.
/// * `bool`: Generates `false` or `true`, each with probability 0.5.
/// * Floating point types (`f32` and `f64`): Uniformly distributed in the
///   half-open range `[0, 1)`. See notes below.
/// * Wrapping integers (`Wrapping<T>`), besides the type identical to their
///   normal integer variants.
///
/// Also supported is the generation of the following
/// compound types where all component types are supported:
///
/// *   Tuples (up to 12 elements): each element is generated sequentially.
/// *   Arrays (up to 32 elements): each element is generated sequentially;
///     see also [`Rng::fill`] which supports arbitrary array length for integer
///     types and tends to be faster for `u32` and smaller types.
/// *   `Option<T>` first generates a `bool`, and if true generates and returns
///     `Some(value)` where `value: T`, otherwise returning `None`.
pub fn random<T>() -> T
where
    Standard: Distribution<T>,
{
    percpu_rng().gen()
}
