/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! /dev/fuse

#![allow(dead_code)]
use alloc::sync::Arc;
use core::sync::atomic::{AtomicI32, Ordering};

use alloc::vec;
use alloc::vec::Vec;
use axfs_vfs::{VfsNodeAttr, VfsNodeOps, VfsNodePerm, VfsNodeRef, VfsNodeType, VfsResult};
use log::*;
use spin::Mutex;
use spinlock::SpinNoIrq;

/// A global flag to indicate the state of the FUSE device.
pub static FUSEFLAG: AtomicI32 = AtomicI32::new(0);
lazy_static::lazy_static! {
    /// vector to store data for FUSE operations.
    pub static ref FUSE_VEC: Arc<SpinNoIrq<Vec<u8>>> = Arc::new(SpinNoIrq::new(Vec::new()));
}

/// A device behaves like `/dev/fuse`.
///
/// It always transmits to the daemon in user space.
pub struct FuseDev {
    data: Mutex<Vec<u8>>,
}

impl Default for FuseDev {
    fn default() -> Self {
        Self::new()
    }
}

impl FuseDev {
    /// Create a new instance.
    pub fn new() -> Self {
        debug!("fuse_dev new here...");
        Self {
            data: Mutex::new(vec![0; 1e8 as usize]),
        }
    }
}

impl VfsNodeOps for FuseDev {
    fn open(&self) -> VfsResult<Option<VfsNodeRef>> {
        debug!("fuse_dev open here...");
        Ok(None)
    }

    fn get_attr(&self) -> VfsResult<VfsNodeAttr> {
        debug!("fuse_dev get_attr here...");
        Ok(VfsNodeAttr::new(
            0,
            VfsNodePerm::default_file(),
            VfsNodeType::CharDevice,
            0,
            0,
        ))
    }

    fn read_at(&self, offset: u64, buf: &mut [u8]) -> VfsResult<usize> {
        debug!(
            "fuse_dev read buf len: {:?} at pos: {:?}",
            buf.len(),
            offset
        );

        let mut flag;

        flag = FUSEFLAG.load(Ordering::SeqCst);
        if flag > 100 {
            debug!("flag in read__ is {:?}, should back to fuse_node.", flag);
            FUSEFLAG.store(-flag, Ordering::Relaxed);
        }

        loop {
            flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag > 0 {
                debug!("flag _read_ is set to {:?},, exiting loop. hhh", flag);
                break;
            }
        }

        let mut vec = FUSE_VEC.lock();
        let vec_len = vec.len();
        buf[..vec_len].copy_from_slice(&vec[..vec_len]);
        debug!("Fusevec _read_ len: {:?}, vec: {:?}", vec.len(), vec);
        vec.clear();

        Ok(vec_len)
    }

    fn write_at(&self, offset: u64, buf: &[u8]) -> VfsResult<usize> {
        debug!(
            "fuse_dev writes buf len: {:?} at pos: {:?}, buf: {:?}",
            buf.len(),
            offset,
            buf
        );

        let mut flag;

        loop {
            flag = FUSEFLAG.load(Ordering::SeqCst);
            if flag > 0 {
                debug!("Fuseflag _write_ is set to {:?},, exiting loop. yyy", flag);
                break;
            }
        }

        let mut vec = FUSE_VEC.lock();
        vec.extend_from_slice(buf);
        debug!("Fusevec _write_ len: {:?}, vec: {:?}", vec.len(), vec);

        FUSEFLAG.store(flag + 100, Ordering::Relaxed);

        Ok(buf.len())
    }

    fn truncate(&self, _size: u64) -> VfsResult {
        Ok(())
    }

    axfs_vfs::impl_vfs_non_dir_default! {}
}
