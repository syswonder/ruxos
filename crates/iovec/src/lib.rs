/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
//! # Scatter/Gather I/O Vector Library
//!
//! This crate provides efficient implementations of scatter/gather I/O operations using
//! I/O vectors (similar to POSIX iovec). It's designed for systems programming where
//! performance and memory efficiency are critical.
//!
//! ## Key Features
//!
//! - **Zero-copy operations**: Avoid unnecessary data copying between buffers
//! - **Small-vector optimization**: Stores single I/O vectors on the stack
//! - **Scatter/gather support**: Read/write across multiple disjoint buffers
//!
//! ## Core Concepts
//!
//! 1. **`IoVector`**: Describes a single memory region (base + length)
//! 2. **`IoVecs`**: Collection of vectors (stack-allocated when possible)
//! 3. **`IoVecsInput`**: For scatter reads (reading into multiple buffers)
//! 4. **`IoVecsOutput`**: For gather writes (writing from multiple buffers)
//!
//! ## Usage Examples
//!
//! ### Writing to multiple buffers
//! ```rust
//! use iovec::{IoVecsOutput, IoVector};
//! use smallvec::smallvec;
//!
//! let mut buf1 = [0u8; 5];
//! let mut buf2 = [0u8; 5];
//!
//! let iovecs = smallvec![
//!     IoVector { base: buf1.as_mut_ptr() as usize, len: 5 },
//!     IoVector { base: buf2.as_mut_ptr() as usize, len: 5 }
//! ];
//!
//! let mut output = IoVecsOutput::from_iovecs(iovecs);
//! let data = [1, 2, 3, 4, 5, 6, 7, 8];
//! let written = output.write(&data);
//! ```
//!
//! ### Reading from multiple buffers
//! ```rust
//! use iovec::{IoVecsInput, IoVector};
//! use smallvec::smallvec;
//! let buf1 = [1u8, 2, 3];
//! let buf2 = [4u8, 5, 6];
//!
//! let iovecs = smallvec![
//!     IoVector { base: buf1.as_ptr() as usize, len: 3 },
//!     IoVector { base: buf2.as_ptr() as usize, len: 3 }
//! ];
//!
//! let input = IoVecsInput::from_iovecs(iovecs);
//! let collected: Vec<u8> = input.read_to_vec(5); // Gets [1, 2, 3, 4, 5]
//! ```
#![cfg_attr(not(test), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use core::{cmp::min, ptr::copy_nonoverlapping};
use smallvec::smallvec;
use smallvec::SmallVec;

type Addr = usize;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
/// Describes a region of memory, beginning at `base` address and with the size of `len` bytes.
pub struct IoVector {
    /// memory begin
    pub base: Addr,
    /// vector size
    pub len: usize,
}

impl IoVector {
    /// Checks if the buffer is empty (zero length)
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Advances the buffer position by reducing length and moving base forward
    pub fn advance(&mut self, length: usize) {
        debug_assert!(
            length <= self.len,
            "Cannot advance more than the length of the IoVec"
        );
        self.base += length;
        self.len -= length;
    }
}

impl<'a> IoVector {
    /// Converts to an immutable byte slice reference
    /// # Safety
    /// Caller must ensure the memory is valid and accessible
    pub fn into_read_buf(self) -> &'a [u8] {
        unsafe { core::slice::from_raw_parts(self.base as *const u8, self.len) }
    }

    /// Converts to a mutable byte slice reference  
    /// # Safety
    /// Caller must ensure the memory is valid and writable
    pub fn into_write_buf(self) -> &'a mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.base as *mut u8, self.len) }
    }
}

/// Collection of I/O vectors with small-array optimization:
/// - Stores 1 vector inline (no heap allocation)
/// - Allocates on heap when more vectors are needed
pub type IoVecs = SmallVec<[IoVector; 1]>;

/// Reads I/O vectors from a pointer (C-style iovec array)
/// # Safety
/// Caller must ensure the pointer and count are valid
pub fn read_iovecs_ptr(iovptr: Addr, iovcnt: usize) -> IoVecs {
    if iovcnt == 0 {
        return SmallVec::new();
    }
    let iovecs = unsafe { core::slice::from_raw_parts(iovptr as *const IoVector, iovcnt) };
    iovecs.iter().copied().collect()
}

/// Output buffer collection for scatter/gather writes
pub struct IoVecsOutput {
    /// I/O vectors (stored in reverse order for efficient popping)
    iovecs: IoVecs,
    /// Total remaining capacity across all buffers
    avaliable: usize,
    /// Total bytes written so far
    bytes_written: usize,
}

impl IoVecsOutput {
    /// Creates from a set of I/O vectors
    pub fn from_iovecs(mut iovecs: IoVecs) -> Self {
        // revserse the iovecs to make it easier to pop from the end
        iovecs.reverse();
        let mut avaliable = 0;
        for iovec in iovecs.iter() {
            avaliable += iovec.len;
        }
        Self {
            iovecs,
            avaliable,
            bytes_written: 0,
        }
    }

    /// Creates from a single mutable buffer
    pub fn from_single_buffer(buf: &mut [u8]) -> Self {
        Self::from_iovecs(smallvec![IoVector {
            base: buf.as_mut_ptr() as usize,
            len: buf.len()
        }])
    }

    /// Returns total remaining writable capacity
    pub fn avaliable(&self) -> usize {
        self.avaliable
    }

    /// Writes data sequentially across buffers
    /// Returns number of bytes actually written
    pub fn write(&mut self, src: &[u8]) -> usize {
        let mut bytes_written = 0;
        let mut remain_bytes_to_write = src.len();
        let mut src_ptr = src.as_ptr() as *mut u8;
        while let Some(mut iovec) = self.iovecs.pop() {
            if iovec.is_empty() {
                continue;
            }
            let bytes_len = min(remain_bytes_to_write, iovec.len);
            unsafe { copy_nonoverlapping(src_ptr, iovec.base as *mut u8, bytes_len) };
            src_ptr = unsafe { src_ptr.add(bytes_len) };
            iovec.advance(bytes_len);
            self.avaliable -= bytes_len;
            self.bytes_written += bytes_len;
            remain_bytes_to_write -= bytes_len;
            bytes_written += bytes_len;
            if !iovec.is_empty() {
                self.iovecs.push(iovec);
                break;
            }
        }
        bytes_written
    }
}

/// Input buffer collection for scatter/gather reads
pub struct IoVecsInput {
    iovecs: IoVecs,
}

impl IoVecsInput {
    /// Creates from a set of I/O vectors
    pub fn from_iovecs(iovecs: IoVecs) -> Self {
        Self { iovecs }
    }

    /// Creates from a single buffer
    pub fn from_single_buffer(buf: &[u8]) -> Self {
        Self::from_iovecs(smallvec![IoVector {
            base: buf.as_ptr() as usize,
            len: buf.len()
        }])
    }

    /// Returns total readable length across all buffers
    pub fn total_len(&self) -> usize {
        let mut total_len = 0;
        for iov in self.iovecs.iter() {
            total_len += iov.len;
        }
        total_len
    }

    /// Reads data into a vector (up to maxlen bytes)
    pub fn read_to_vec(&self, maxlen: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(maxlen.min(self.total_len()));
        let mut remaining = maxlen;

        for iov in self.iovecs.iter() {
            if remaining == 0 {
                break;
            }
            let read_len = iov.len.min(remaining);
            let bytes = unsafe {
                let ptr = iov.base as *const u8;
                core::slice::from_raw_parts(ptr, read_len)
            };
            result.extend_from_slice(bytes);
            remaining -= read_len;
        }

        result
    }

    /// Returns an iterator over buffer slices
    pub fn as_slices(&self) -> impl Iterator<Item = &[u8]> + '_ {
        self.iovecs
            .iter()
            .map(|iov| unsafe { core::slice::from_raw_parts(iov.base as *const u8, iov.len) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::mem::MaybeUninit;

    fn create_buffer(size: usize) -> (&'static mut [MaybeUninit<u8>], Addr) {
        let buf = Box::leak(vec![MaybeUninit::uninit(); size].into_boxed_slice());
        let ptr = buf.as_ptr() as usize;
        (buf, ptr)
    }

    #[test]
    fn test_single_iovec_full_write() {
        let (buf, ptr) = create_buffer(5);
        let iovecs = SmallVec::from_vec(vec![IoVector { base: ptr, len: 5 }]);
        let mut output = IoVecsOutput::from_iovecs(iovecs);

        let src = [1, 2, 3, 4, 5];
        let written = output.write(&src);

        assert_eq!(written, 5);
        assert_eq!(output.bytes_written, 5);
        assert_eq!(output.avaliable, 0);

        let filled = unsafe { &*(buf as *const _ as *const [u8; 5]) };
        assert_eq!(filled, &[1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_multiple_iovecs_partial_write() {
        let (buf1, ptr1) = create_buffer(5);
        let (buf2, ptr2) = create_buffer(5);

        let iovecs = SmallVec::from_vec(vec![
            IoVector { base: ptr1, len: 5 },
            IoVector { base: ptr2, len: 5 },
        ]);
        let mut output = IoVecsOutput::from_iovecs(iovecs);

        let src = [1, 2, 3, 4, 5, 6, 7, 8];
        let written = output.write(&src);

        assert_eq!(written, 8);
        assert_eq!(output.bytes_written, 8);
        assert_eq!(output.avaliable, 2);

        let filled1 = unsafe { &*(buf1 as *const _ as *const [u8; 5]) };
        let filled2 = unsafe { &*(buf2 as *const _ as *const [u8; 3]) };
        assert_eq!(filled1, &[1, 2, 3, 4, 5]);
        assert_eq!(filled2, &[6, 7, 8]);
    }

    #[test]
    fn test_write_more_than_available() {
        let (buf, ptr) = create_buffer(3);
        let iovecs = SmallVec::from_vec(vec![IoVector { base: ptr, len: 3 }]);
        let mut output = IoVecsOutput::from_iovecs(iovecs);

        let src = [1, 2, 3, 4, 5];
        let written = output.write(&src);

        assert_eq!(written, 3);
        assert_eq!(output.bytes_written, 3);
        assert_eq!(output.avaliable, 0);
        let filled = unsafe { &*(buf as *const _ as *const [u8; 3]) };
        assert_eq!(filled, &[1, 2, 3]);
    }

    #[test]
    fn test_empty_iovecs() {
        let iovecs = SmallVec::new();
        let mut output = IoVecsOutput::from_iovecs(iovecs);
        let src = [1, 2, 3];
        let written = output.write(&src);
        assert_eq!(written, 0);
    }

    #[test]
    fn test_partial_advance() {
        let (buf, ptr) = create_buffer(5);
        let mut iovec = IoVector { base: ptr, len: 5 };

        iovec.advance(2);
        assert_eq!(iovec.len, 3);
        assert_eq!(iovec.base, ptr + 2);

        let src = [9, 8, 7];
        unsafe { copy_nonoverlapping(src.as_ptr(), iovec.base as *mut u8, 3) };
        let filled = unsafe { &*(buf as *const _ as *const [u8; 5]) };
        assert_eq!(&filled[2..], &[9, 8, 7]);
    }

    #[test]
    #[should_panic]
    fn test_advance_panic() {
        let (_, ptr) = create_buffer(5);
        let mut iovec = IoVector { base: ptr, len: 3 };
        iovec.advance(5);
    }

    #[test]
    fn test_multiple_writes_with_partial_fills() {
        let (buf1, ptr1) = create_buffer(4); // 长度4
        let (buf2, ptr2) = create_buffer(5); // 长度5
        let (buf3, ptr3) = create_buffer(5); // 长度5

        let iovecs = SmallVec::from_vec(vec![
            IoVector { base: ptr1, len: 4 },
            IoVector { base: ptr2, len: 5 },
            IoVector { base: ptr3, len: 5 },
        ]);
        let mut output = IoVecsOutput::from_iovecs(iovecs);
        assert_eq!(output.avaliable, 4 + 5 + 5);

        let src1 = [1, 2, 3, 4, 5];
        let written = output.write(&src1);
        assert_eq!(written, 5); // 4+1
        assert_eq!(output.bytes_written, 5);
        assert_eq!(output.avaliable, 4 + 5 + 5 - 5);

        let filled1 = unsafe { &*(buf1 as *const _ as *const [u8; 4]) };
        let filled2_part = unsafe { &*(buf2 as *const _ as *const [u8; 1]) };
        assert_eq!(filled1, &[1, 2, 3, 4]);
        assert_eq!(filled2_part, &[5]);

        let src2 = [6, 7, 8];
        let written = output.write(&src2);
        assert_eq!(written, 3);
        assert_eq!(output.bytes_written, 5 + 3);
        assert_eq!(output.avaliable, 4 + 5 + 5 - 5 - 3);

        let filled2 = unsafe { &*(buf2 as *const _ as *const [u8; 5]) };
        assert_eq!(&filled2[0..4], &[5, 6, 7, 8]);

        let src3 = [9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        let written = output.write(&src3);
        assert_eq!(written, 6);
        assert_eq!(output.bytes_written, 5 + 3 + 6);
        assert_eq!(output.avaliable, 0); // 14 - 14

        let filled2_remain = unsafe { &*(buf2.as_ptr().add(4) as *const u8) };
        assert_eq!(*filled2_remain, 9);
        let filled3 = unsafe { &*(buf3 as *const _ as *const [u8; 5]) };
        assert_eq!(filled3, &[10, 11, 12, 13, 14]); // write 5 bytes

        let src4 = [99, 100, 101];
        let written = output.write(&src4);
        assert_eq!(written, 0);
    }
}
