/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeType, VfsResult};
use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

static SEED: AtomicU64 = AtomicU64::new(0xae_f3);

/// A random device behaves like `/dev/random`.
///
/// It always returns a chunk of random bytes when read, and all writes are discarded.
pub struct RandomDev;

/// Returns a 32-bit unsigned pseudo random interger using LCG.
fn rand_lcg32() -> u32 {
    let new_seed = SEED
        .load(SeqCst)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

impl VfsNodeOps for RandomDev {
    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        Ok(VfsNodeAttr::new(
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn read_at(&self, _offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        let len = buf.len() >> 2;
        let remainder = buf.len() & 0x3;
        for i in 0..len {
            let random = rand_lcg32();
            let start_idx = i * 4;
            // MSB
            buf[start_idx..start_idx + 4].copy_from_slice(random.to_be_bytes().as_ref());
        }

        let random = rand_lcg32();
        buf[len * 4..].copy_from_slice(random.to_be_bytes()[..remainder].as_ref());

        Ok(buf.len())
    }

    fn write_at(&self, _offset: u64, buf: &[u8]) -> VfsResult<usize> {
        Ok(buf.len())
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    axfs_vfs::impl_vfs_non_dir_default! {}
}
