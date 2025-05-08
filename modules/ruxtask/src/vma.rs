/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Virtual Memory Area (VMA) data structure.
//!
//! This module provides data structures for virtual memory area (VMA) management.
//! TODO: use `Mutex` to replace `SpinNoIrq` to make it more efficient.

use crate::current;
use crate::fs::get_file_like;
use crate::TaskId;
use alloc::{collections::BTreeMap, sync::Arc};
use axalloc::global_allocator;
use memory_addr::PhysAddr;
use ruxfs::File;
use ruxhal::mem::phys_to_virt;

#[cfg(feature = "fs")]
use alloc::vec::Vec;
#[cfg(feature = "fs")]
use memory_addr::PAGE_SIZE_4K;

use spinlock::SpinNoIrq;

// use `used_fs` instead of `#[cfg(feature = "fs")]{}` to cancel the scope of code.
#[cfg(feature = "fs")]
macro_rules! used_fs {
     ($($code:tt)*) => {$($code)*};
 }

#[cfg(not(feature = "fs"))]
macro_rules! used_fs {
    ($($code:tt)*) => {};
}

// TODO: move defination of `SWAP_MAX` and `SWAP_PATH` from const numbers to `ruxconfig`.
used_fs! {
    // pub(crate) const SWAP_MAX: usize = 1024 * 1024 * 1024;
    pub(crate) const SWAP_MAX: usize = 0;
    pub(crate) const SWAP_PATH: &str = "swap.raw\0";
    /// record the mapping of swapped out pages.
    pub static SWAPED_MAP: SpinNoIrq<BTreeMap<usize, Arc<SwapInfo>>> = SpinNoIrq::new(BTreeMap::new()); // Vaddr => (page_size, offset_at_swaped)
    lazy_static::lazy_static! {
        /// swap file for swapping out pages.
        pub static ref SWAP_FILE: Arc<File> = open_swap_file(SWAP_PATH);
        /// bitmap for free pages in swap file.
        pub static ref BITMAP_FREE: SpinNoIrq<Vec<usize>> = SpinNoIrq::new((0..SWAP_MAX).step_by(PAGE_SIZE_4K).collect());
    }
}

/// open target file
#[cfg(feature = "fs")]
fn open_swap_file(filename: &str) -> Arc<File> {
    use crate::fs::absolute_path;
    use ruxfdtable::OpenFlags;

    let opt = OpenFlags::O_RDWR | OpenFlags::O_APPEND | OpenFlags::O_CREAT;
    let path = absolute_path(filename).unwrap();
    ruxfs::fops::open_file_like(&path, opt)
        .expect("create swap file failed")
        .into_any()
        .downcast::<File>()
        .expect("create swap file failed")
}

/// Data structure for file mapping.
#[derive(Clone)]
pub struct FileInfo {
    /// file that the mapping is backed by
    pub file: Arc<File>,
    /// offset in the file
    pub offset: usize,
    /// size of the mapping
    pub size: usize,
}

/// Data structure for information of mapping.
pub struct PageInfo {
    /// physical address of the page
    pub paddr: PhysAddr,
    #[cfg(feature = "fs")]
    /// file that the mapping is backed by
    pub mapping_file: Option<FileInfo>,
}

/// Data structure for swaping out a page in a file.
#[derive(Debug, Clone)]
pub struct SwapInfo {
    /// offset in the swap file
    pub offset: usize,
}

impl From<usize> for SwapInfo {
    fn from(value: usize) -> Self {
        SwapInfo { offset: value }
    }
}

/// Data structure for mmap for a specific process.
pub struct MmapStruct {
    /// virtual memory area list
    pub vma_map: SpinNoIrq<BTreeMap<usize, Vma>>,
    /// page that already loaded into memory
    pub mem_map: SpinNoIrq<BTreeMap<usize, Arc<PageInfo>>>,
    /// pages that swapped out into swap file or disk
    pub swaped_map: SpinNoIrq<BTreeMap<usize, Arc<SwapInfo>>>,
}

/// clone data structure for MmapStruct (when forking).
impl Clone for MmapStruct {
    fn clone(&self) -> Self {
        Self {
            vma_map: SpinNoIrq::new(self.vma_map.lock().clone()),
            mem_map: SpinNoIrq::new(self.mem_map.lock().clone()),
            swaped_map: SpinNoIrq::new(self.swaped_map.lock().clone()),
        }
    }
}

// release memory of a page in swaping file
#[cfg(feature = "fs")]
impl Drop for SwapInfo {
    fn drop(&mut self) {
        BITMAP_FREE.lock().push(self.offset);
    }
}

// release memory of a page in memory
impl Drop for PageInfo {
    fn drop(&mut self) {
        // use `global_allocator()` to dealloc pages.
        global_allocator().dealloc_pages(phys_to_virt(self.paddr).as_usize(), 1);
    }
}

/// Data structure for mapping [start_addr, end_addr) with meta data.
#[derive(Clone)]
pub struct Vma {
    /// start address of the mapping
    pub start_addr: usize,
    /// end address of the mapping
    pub end_addr: usize,
    /// mmap size of the mapping
    pub size: usize,
    /// file that the mapping is backed by
    pub file: Option<Arc<File>>,
    /// offset in the file
    pub offset: usize,
    /// size of the mapping
    pub prot: u32,
    /// flags of the mapping
    pub flags: u32,
    /// process that the mapping belongs to
    pub from_process: TaskId,
}

impl MmapStruct {
    /// Create a new `MmapStruct` instance.
    pub const fn new() -> Self {
        Self {
            vma_map: SpinNoIrq::new(BTreeMap::new()),
            mem_map: SpinNoIrq::new(BTreeMap::new()),
            swaped_map: SpinNoIrq::new(BTreeMap::new()),
        }
    }
}

/// Impl for Vma.
impl Vma {
    /// Create a new `Vma` instance.
    pub fn new(_fid: i32, offset: usize, prot: u32, flags: u32) -> Self {
        // #[cfg(feature = "fs")]
        let file = if _fid < 0 {
            None
        } else {
            let f = get_file_like(_fid).expect("invaild fd for vma");
            Some(
                f.clone()
                    .into_any()
                    .downcast::<File>()
                    .expect("should be effective fid"),
            )
        };
        Vma {
            start_addr: 0,
            end_addr: 0,
            size: 0,
            // #[cfg(feature = "fs")]
            file,
            offset,
            flags,
            prot,
            from_process: current().id(),
        }
    }

    /// Clone a new `Vma` instance.
    pub fn clone_from(vma: &Vma, start_addr: usize, end_addr: usize) -> Self {
        Vma {
            start_addr,
            end_addr,
            size: vma.size,
            // #[cfg(feature = "fs")]
            file: vma.file.clone(),
            offset: vma.offset,
            prot: vma.prot,
            flags: vma.prot,
            from_process: current().id(),
        }
    }
}
