/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Runtime library of [Ruxos](https://github.com/syswonder/ruxos).
//!
//! This module provides the implementation of kernel-level random
//! number generators (RNGs), especially the per-CPU RNG type. It also
//! enables the usage of random exponential backoff strategy in spinlocks.
//!
//! # Cargo Features
//!
//! - `easy-spin`: Use a alternate, extremely simple RNG for backoff in
//!   spinlocks, instead of the default per-CPU RNG. This may increase
//!   performance when the lock is not highly contended.
//!
//! All the features are optional and disabled by default.

#![cfg_attr(not(test), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]

/// Defines the per-CPU RNG.
pub mod rng;

mod spin_rand;

/// Initializes the per-CPU RNGs on the given CPU.
pub fn init(cpuid: usize) {
    rng::init(cpuid);
}
