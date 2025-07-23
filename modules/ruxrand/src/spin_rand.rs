/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::{
    fmt::Debug,
    sync::atomic::{AtomicUsize, Ordering},
};

use rand::RngCore;
use spinlock::{Backoff, Relax};

#[cfg(feature = "easy-spin")]
type SpinRng = EasyRng;

#[cfg(not(feature = "easy-spin"))]
type SpinRng = crate::rng::PercpuRng;

#[inline(always)]
fn exp_rand_backoff(current_limit: &mut u32, max: u32) {
    let limit = *current_limit;
    *current_limit = max.max(limit);

    let mut rng = SpinRng::default();
    // It is more "correct" to use `rng.gen_range(0..limit)`,
    // but since `limit` would only be powers of two, a simple
    // modulo would also keep the distribution uniform as long
    // as `rng.next_u32()` keeps a uniform distribution on `u32`.
    let delay = rng.next_u32() % limit;
    for _ in 0..delay {
        core::hint::spin_loop();
    }
}

/// Calls [`core::hint::spin_loop`] random times within an exponentially grown limit
/// when backoff/relax is required. The random number is generated using [`RngCore::next_u32`],
/// and the actual rng used is controlled by the `easy-spin` feature.
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

#[derive(Clone, Default)]
struct EasyRng;

impl Debug for EasyRng {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let state = EASY_RNG_STATE.load(Ordering::Relaxed);
        f.debug_struct("EasyRng").field("state", &state).finish()
    }
}

impl RngCore for EasyRng {
    fn next_u32(&mut self) -> u32 {
        easy_rng()
    }

    fn next_u64(&mut self) -> u64 {
        easy_rng() as _
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        dest.fill_with(|| easy_rng() as _)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        dest.fill_with(|| easy_rng() as _);
        Ok(())
    }
}

static EASY_RNG_STATE: AtomicUsize = AtomicUsize::new(0);

fn easy_rng() -> u32 {
    const RANDOM_RANDOM_LIST: [u8; 64] = [
        9, 7, 13, 0, 15, 2, 14, 1, 14, 14, 11, 3, 13, 11, 12, 10, 3, 6, 8, 1, 2, 0, 12, 12, 13, 2,
        9, 5, 3, 10, 6, 1, 15, 9, 6, 12, 9, 7, 4, 7, 4, 8, 11, 7, 0, 1, 2, 10, 15, 6, 5, 3, 0, 5,
        14, 4, 4, 13, 15, 8, 5, 10, 8, 11,
    ];

    let idx = EASY_RNG_STATE.fetch_add(1, Ordering::Relaxed) % RANDOM_RANDOM_LIST.len();
    RANDOM_RANDOM_LIST[idx] as _
}
