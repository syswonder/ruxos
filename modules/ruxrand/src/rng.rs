/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use lazy_init::LazyInit;
use log::debug;
use percpu::def_percpu;
use rand::{distributions::Standard, prelude::*};
use rand_xoshiro::Xoshiro256StarStar;
use ruxdriver::prelude::*;

#[allow(clippy::unusual_byte_groupings)]
const RNG_SEED: u64 = 0xBAD_C0FFEE_0DD_F00D;

/// A per-CPU random number generator type that can be either a Xoshiro RNG or a device RNG.
pub enum PercpuRngType {
    /// A Xoshiro256StarStar pseudo RNG.
    Xoshiro(Xoshiro256StarStar),
    /// A device RNG, which is typically used for hardware RNG devices.
    Device(AxRngDevice),
}

#[def_percpu]
pub(crate) static PERCPU_RNG: LazyInit<PercpuRngType> = LazyInit::new();

/// Initializes the random number generator device.
pub fn init_dev(rng_dev: AxRngDevice, cpuid: usize) {
    // Initialize the per-CPU RNG for the given CPU ID.
    // This is a placeholder function and should be implemented
    // to initialize the RNG for the specific CPU.
    debug!("Initializing per-CPU RNG for CPU ID: {cpuid}");
    PERCPU_RNG.with_current(|percpu_ref| {
        percpu_ref.init_by(PercpuRngType::Device(rng_dev));
    });
}

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
        let mut rng = Xoshiro256StarStar::seed_from_u64(RNG_SEED);
        for _ in 0..cpuid {
            rng.long_jump();
        }

        percpu_ref.init_by(PercpuRngType::Xoshiro(rng));
    });
}

/// Returns a 32-bit random number using the per-CPU RNG.
// TODO: Implement the actual RNG initialization logic
pub fn percpu_rng() {
    todo!("Implement the per-CPU RNG initialization logic");
}

/// A simple random number generator that uses the per-CPU RNG.
pub struct PercpuRng;

/// Generates a random value of type `T`
pub fn random<T>() -> T
where
    Standard: rand::distributions::Distribution<T>,
{
    let size = core::mem::size_of::<T>();
    let mut buf = [0u8; 64];
    assert!(size <= 64, "Requested type is too big");

    let dst = &mut buf[..size];
    let rng: *mut PercpuRngType =
        unsafe { PERCPU_RNG.current_ref_mut_raw().get_mut_unchecked() as *mut _ };
    match unsafe { &mut *rng } {
        PercpuRngType::Device(ref mut dev) => {
            let _ = dev.request_entropy(dst);
            unsafe { core::ptr::read_unaligned(dst.as_ptr() as *const T) }
        }
        PercpuRngType::Xoshiro(ref mut rng) => rng.gen(),
    }
}

/// Requests entropy from the RNG and fills the provided buffer.
pub fn request_entropy(dst: &mut [u8]) -> DevResult<usize> {
    let rng: *mut PercpuRngType =
        unsafe { PERCPU_RNG.current_ref_mut_raw().get_mut_unchecked() as *mut _ };
    match unsafe { &mut *rng } {
        PercpuRngType::Device(ref mut dev) => dev.request_entropy(dst),
        PercpuRngType::Xoshiro(ref mut rng) => {
            rng.fill_bytes(dst);
            Ok(dst.len())
        }
    }
}

/// Returns a 32-bit unsigned random integer.
pub fn next_u32() -> u32 {
    random::<u32>()
}

/// Returns a 64-bit unsigned random integer.
pub fn next_u64() -> u64 {
    random::<u64>()
}

/// Fills the provided buffer with random bytes.
pub fn fill_bytes(dest: &mut [u8]) {
    request_entropy(dest).expect("Failed to fill bytes from RNG");
}

/// Attempts to fill the provided buffer with random bytes, returning an error if it fails.
pub fn try_fill_bytes(_dest: &mut [u8]) -> Result<(), rand::Error> {
    todo!()
}
