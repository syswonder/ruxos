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
    hash::{Hash, Hasher},
    sync::atomic::{self, AtomicI32},
};

use ahash::AHasher;
use alloc::vec::Vec;

use ruxtask::WaitQueueWithMetadata;

use super::BUCKET_MASK;

#[derive(Clone, Copy)]
pub(crate) struct FutexKey {
    key: usize,
    bitset: u32,
}

impl Debug for FutexKey {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("FutexKey")
            .field("key", &format_args!("{:#x}", self.key))
            .field("bitset", &format_args!("{:#x}", self.bitset))
            .finish()
    }
}

pub(crate) type FutexBucket = WaitQueueWithMetadata<FutexKey>;

pub(crate) struct FutexVec {
    pub(crate) buckets: Vec<FutexBucket>,
}

impl FutexKey {
    /// Create futex key from its address and a bitset.
    ///
    /// Note that `addr` or `self.key` is actually a [`PhysAddr`](memory_addr::PhysAddr) pointing to a [`i32`]
    /// but while RuxOS only supports single addr space, we'd like to treat it as a normal pointer.
    pub fn new(addr: *const i32, bitset: u32) -> Self {
        Self {
            key: addr as usize,
            bitset,
        }
    }

    /// Load the key value, atomically.
    #[inline]
    pub fn load_val(&self) -> i32 {
        let ptr = self.key as *const AtomicI32;
        unsafe { (*ptr).load(atomic::Ordering::SeqCst) }
    }

    /// Return the address that this futex key references.
    #[inline]
    pub fn addr(&self) -> usize {
        self.key
    }

    #[inline]
    pub fn bitset(&self) -> u32 {
        self.bitset
    }
}

impl PartialEq for FutexKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl FutexVec {
    pub fn new(size: usize) -> Self {
        let buckets = (0..size)
            .map(|_| WaitQueueWithMetadata::new())
            .collect::<Vec<_>>();
        Self { buckets }
    }

    pub fn get_bucket(&self, key: FutexKey) -> (usize, &FutexBucket) {
        let hash = {
            // this addr should be aligned as a `*const u32`, which is this multiples of 4,
            // so ignoring the last 2 bits is fine
            let addr = key.addr() >> 2;
            let mut hasher = AHasher::default();
            addr.hash(&mut hasher);
            hasher.finish() as usize
        };
        let idx = BUCKET_MASK & hash;
        (idx, &self.buckets[idx])
    }
}
