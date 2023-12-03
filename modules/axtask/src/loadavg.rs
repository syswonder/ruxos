/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::sync::atomic::{AtomicU64, Ordering};

use crate::AVENRUN;

/// bits to shift fixed point
const FSHIFT: u64 = 16;
/// fixed point
const FIXED_1: u64 = 1 << FSHIFT;
/// update AVENRUN per 5 seconds
const LOAD_FREQ: u64 = 5 * axhal::time::NANOS_PER_SEC + 1;

/* 1/exp(5sec/1min) as fixed-point */
/* 1/exp(5sec/5min) */
/* 1/exp(5sec/15min) */
const EXP: [u64; 3] = [1884, 2014, 2037];

/// count of idle ticks
static mut IDLE_CNT: AtomicU64 = AtomicU64::new(0);
/// count of all ticks
static mut ALL_CNT: AtomicU64 = AtomicU64::new(0);
/// last update time
static mut LAST_UPDATE: AtomicU64 = AtomicU64::new(0);

/*
 * a1 = a0 * e + a * (1 - e)
 */
fn calc_load(load: u64, exp: u64, active: u64) -> u64 {
    let mut newload: u64 = load * exp + active * (FIXED_1 - exp);
    if active >= load {
        newload += FIXED_1 - 1;
    }
    newload / FIXED_1
}

/*
 * calc_load_tick - update the avenrun load
 *
 * Called from the scheduler_timer_tick.
 */
pub(crate) fn calc_load_tick(is_idle: bool) {
    if is_idle {
        unsafe {
            IDLE_CNT.fetch_add(1, Ordering::Relaxed);
        }
    }
    unsafe {
        ALL_CNT.fetch_add(1, Ordering::Relaxed);
    }

    let curr = axhal::time::current_time_nanos();

    if curr - unsafe { LAST_UPDATE.load(Ordering::Relaxed) } < LOAD_FREQ {
        return;
    }
    let idle_cnt;
    let all_cnt;
    unsafe {
        LAST_UPDATE.store(curr, Ordering::Relaxed);
        idle_cnt = IDLE_CNT.load(Ordering::Relaxed);
        IDLE_CNT.store(0, Ordering::Relaxed);
        all_cnt = ALL_CNT.load(Ordering::Relaxed);
        ALL_CNT.store(0, Ordering::Relaxed);
    }
    for i in 0..3 {
        unsafe {
            AVENRUN[i] = calc_load(AVENRUN[i], EXP[i], (all_cnt - idle_cnt) * FIXED_1 / all_cnt);
        }
    }
}
