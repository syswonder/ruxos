//! A circular buffer (ring buffer) implementation for efficient FIFO operations.
#![cfg_attr(not(test), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::cmp;

/// Represents the current state of the ring buffer
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
enum RingBufferState {
    #[default]
    /// Buffer contains no data
    Empty,
    /// Buffer is completely full
    Full,
    /// Buffer has data but isn't full
    Normal,
}

/// A circular buffer implementation using a `Vec<u8>` as backing storage
pub struct RingBuffer {
    /// Underlying data storage
    arr: Vec<u8>,
    // NOTE: When and only when `head` equals `tail`, `state` can only be `Full` or `Empty`.
    /// Index of the next element to read
    head: usize,
    /// Index of the next element to write
    tail: usize,
    /// Current buffer state
    state: RingBufferState,
}

impl RingBuffer {
    /// Creates a new RingBuffer with the specified capacity
    ///
    /// # Arguments
    /// * `len` - Capacity of the buffer (must be greater than 0)
    ///
    /// # Panics
    /// Panics if `len` is 0
    pub fn new(len: usize) -> Self {
        assert!(len > 0, "Buffer length must be positive");
        Self {
            arr: vec![0; len],
            head: 0,
            tail: 0,
            state: RingBufferState::Empty,
        }
    }

    /// Returns true if the buffer contains no data
    pub fn is_empty(&self) -> bool {
        self.state == RingBufferState::Empty
    }

    /// Returns true if the buffer has no free space
    pub fn is_full(&self) -> bool {
        self.state == RingBufferState::Full
    }

    /// Read as much as possible to fill `dst`.
    ///
    /// # Arguments
    /// * `dst` - Destination buffer for read data
    ///
    /// # Returns
    /// Number of bytes actually written
    pub fn read(&mut self, dst: &mut [u8]) -> usize {
        if self.state == RingBufferState::Empty || dst.is_empty() {
            return 0;
        }

        let ret_len;
        let n = self.arr.len();
        if self.head < self.tail {
            ret_len = cmp::min(self.tail - self.head, dst.len());
            dst[..ret_len].copy_from_slice(&self.arr[self.head..self.head + ret_len]);
        } else {
            // also handles full
            ret_len = cmp::min(n - self.head + self.tail, dst.len());
            if ret_len <= (n - self.head) {
                dst[..ret_len].copy_from_slice(&self.arr[self.head..self.head + ret_len]);
            } else {
                let right_len = n - self.head;
                dst[..right_len].copy_from_slice(&self.arr[self.head..]);
                dst[right_len..ret_len].copy_from_slice(&self.arr[..(ret_len - right_len)]);
            }
        }
        self.head = (self.head + ret_len) % n;

        if self.head == self.tail {
            self.state = RingBufferState::Empty;
        } else {
            self.state = RingBufferState::Normal;
        }

        ret_len
    }

    /// Write as much as possible to fill the ring buffer.
    ///
    /// # Arguments
    /// * `src` - Source buffer containing data to write
    ///
    /// # Returns
    /// Number of bytes actually written
    pub fn write(&mut self, src: &[u8]) -> usize {
        if self.state == RingBufferState::Full || src.is_empty() {
            return 0;
        }

        let ret_len;
        let n = self.arr.len();
        if self.head <= self.tail {
            // also handles empty
            ret_len = cmp::min(n - (self.tail - self.head), src.len());
            if ret_len <= (n - self.tail) {
                self.arr[self.tail..self.tail + ret_len].copy_from_slice(&src[..ret_len]);
            } else {
                self.arr[self.tail..].copy_from_slice(&src[..n - self.tail]);
                self.arr[..(ret_len - (n - self.tail))]
                    .copy_from_slice(&src[n - self.tail..ret_len]);
            }
        } else {
            ret_len = cmp::min(self.head - self.tail, src.len());
            self.arr[self.tail..self.tail + ret_len].copy_from_slice(&src[..ret_len]);
        }
        self.tail = (self.tail + ret_len) % n;

        if self.head == self.tail {
            self.state = RingBufferState::Full;
        } else {
            self.state = RingBufferState::Normal;
        }

        ret_len
    }

    /// Removes and returns the next byte from the buffer
    ///
    /// # Returns
    /// `Some(byte)` if available, `None` if buffer is empty
    pub fn dequeue(&mut self) -> Option<u8> {
        if self.is_empty() {
            return None;
        }

        let n = self.arr.len();
        let c = self.arr[self.head];
        self.head = (self.head + 1) % n;
        if self.head == self.tail {
            self.state = RingBufferState::Empty;
        } else {
            self.state = RingBufferState::Normal;
        }
        Some(c)
    }

    /// Adds a single byte to the buffer
    ///
    /// # Arguments
    /// * `byte` - Byte to add to the buffer
    ///
    /// # Returns
    /// `Some(())` if successful, `None` if buffer is full
    pub fn enqueue(&mut self, byte: u8) -> Option<()> {
        if self.is_full() {
            return None;
        }

        let n = self.arr.len();
        self.arr[self.tail] = byte;
        self.tail = (self.tail + 1) % n;
        if self.head == self.tail {
            self.state = RingBufferState::Full;
        } else {
            self.state = RingBufferState::Normal;
        }
        Some(())
    }

    /// Forces a byte into the buffer by overwriting the oldest element if full
    ///
    /// # Arguments
    /// * `byte` - Byte to be inserted into the buffer
    pub fn force_enqueue(&mut self, byte: u8) {
        if self.is_full() {
            self.dequeue();
        }
        self.enqueue(byte);
    }

    /// Clears the buffer, resetting it to an empty state.
    ///
    /// This method resets the read and write pointers to the start of the buffer
    /// and marks the buffer as empty. All existing data in the buffer is logically
    /// discarded, and the full capacity becomes available for new writes.
    pub fn clear(&mut self) {
        self.head = 0;
        self.tail = 0;
        self.state = RingBufferState::Empty;
    }

    /// Reads and removes all available data from the buffer, returning the collected bytes
    pub fn drain(&mut self) -> Vec<u8> {
        let mut ret = vec![0u8; self.available_read()];
        self.read(ret.as_mut_slice());
        ret
    }

    /// Returns the number of bytes available for reading
    pub fn available_read(&self) -> usize {
        match self.state {
            RingBufferState::Empty => 0,
            RingBufferState::Full => self.arr.len(),
            RingBufferState::Normal => {
                if self.head < self.tail {
                    self.tail - self.head
                } else {
                    (self.arr.len() - self.head) + self.tail
                }
            }
        }
    }

    /// Returns the number of bytes available for writing
    pub fn available_write(&self) -> usize {
        self.arr.len() - self.available_read()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let rb = RingBuffer::new(5);
        assert_eq!(rb.arr.len(), 5);
        assert!(rb.is_empty());
        assert!(!rb.is_full());
    }

    #[test]
    fn test_enqueue_dequeue_single() {
        let mut rb = RingBuffer::new(3);
        assert_eq!(rb.enqueue(1), Some(()));
        assert!(!rb.is_empty());
        assert!(!rb.is_full());
        assert_eq!(rb.dequeue(), Some(1));
        assert!(rb.is_empty());
    }

    #[test]
    fn test_full_condition() {
        let mut rb = RingBuffer::new(2);
        assert_eq!(rb.enqueue(1), Some(()));
        assert_eq!(rb.enqueue(2), Some(()));
        assert!(rb.is_full());
        assert_eq!(rb.enqueue(3), None);
    }

    #[test]
    fn test_empty_condition() {
        let mut rb = RingBuffer::new(2);
        assert_eq!(rb.dequeue(), None);
        rb.enqueue(1).unwrap();
        rb.dequeue().unwrap();
        assert_eq!(rb.dequeue(), None);
    }

    #[test]
    fn test_wrap_around() {
        let mut rb = RingBuffer::new(3);
        rb.enqueue(1).unwrap();
        rb.enqueue(2).unwrap();
        rb.enqueue(3).unwrap();
        assert!(rb.is_full());
        assert_eq!(rb.dequeue().unwrap(), 1);
        assert_eq!(rb.dequeue().unwrap(), 2);
        assert_eq!(rb.dequeue().unwrap(), 3);
        assert!(rb.is_empty());

        rb.enqueue(4).unwrap();
        rb.enqueue(5).unwrap();
        assert_eq!(rb.dequeue().unwrap(), 4);
        rb.enqueue(6).unwrap();
        assert_eq!(rb.dequeue().unwrap(), 5);
        assert_eq!(rb.dequeue().unwrap(), 6);
    }

    #[test]
    fn test_read_write_basic() {
        let mut rb = RingBuffer::new(5);
        let data = [1, 2, 3];
        assert_eq!(rb.write(&data), 3);
        assert_eq!(rb.head, 0);
        assert_eq!(rb.tail, 3);

        let mut buf = [0; 5];
        assert_eq!(rb.read(&mut buf), 3);
        assert_eq!(&buf[..3], &[1, 2, 3]);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_read_write_wrap() {
        let mut rb = RingBuffer::new(5);

        assert_eq!(rb.write(&[1, 2, 3, 4]), 4);
        assert_eq!(rb.head, 0);
        assert_eq!(rb.tail, 4);
        assert!(!rb.is_full());

        let mut buf = [0; 3];
        assert_eq!(rb.read(&mut buf), 3);
        assert_eq!(buf, [1, 2, 3]);
        assert_eq!(rb.head, 3);
        assert_eq!(rb.tail, 4);

        assert_eq!(rb.write(&[5, 6, 7]), 3);
        assert_eq!(rb.tail, (4 + 3) % 5);
        assert_eq!(rb.tail, 2);

        let mut buf = [0; 5];
        assert_eq!(rb.read(&mut buf), 4);
        assert_eq!(&buf[..4], &[4, 5, 6, 7]);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_full_read_write() {
        let mut rb = RingBuffer::new(5);
        assert_eq!(rb.write(&[1, 2, 3, 4, 5]), 5);
        assert!(rb.is_full());
        assert_eq!(rb.write(&[6]), 0);
        let mut buf = [0; 5];
        assert_eq!(rb.read(&mut buf), 5);
        assert_eq!(buf, [1, 2, 3, 4, 5]);
        assert!(rb.is_empty());
    }

    #[test]
    fn test_partial_read_write() {
        let mut rb = RingBuffer::new(5);
        assert_eq!(rb.write(&[1, 2]), 2);
        let mut buf = [0; 3];
        assert_eq!(rb.read(&mut buf), 2);
        assert_eq!(&buf[..2], &[1, 2]);
    }

    #[test]
    fn test_buffer_edge_cases() {
        let mut rb = RingBuffer::new(1);
        assert!(rb.is_empty());
        rb.enqueue(42).unwrap();
        assert!(rb.is_full());
        assert_eq!(rb.dequeue(), Some(42));
        assert!(rb.is_empty());
    }

    #[test]
    fn test_complex_operations() {
        let mut rb = RingBuffer::new(5);
        rb.write(&[1, 2, 3]);
        let mut buf = [0; 2];
        rb.read(&mut buf);
        rb.write(&[4, 5, 6, 7]);
        let mut buf = [0; 5];
        assert_eq!(rb.read(&mut buf), 5);
    }

    #[test]
    fn test_state_transitions() {
        let mut rb = RingBuffer::new(3);
        // Empty -> Normal
        rb.enqueue(1).unwrap();
        assert_eq!(rb.state, RingBufferState::Normal);
        // Normal -> Full
        rb.enqueue(2).unwrap();
        rb.enqueue(3).unwrap();
        assert_eq!(rb.state, RingBufferState::Full);
        // Full -> Normal
        rb.dequeue().unwrap();
        assert_eq!(rb.state, RingBufferState::Normal);
        // Normal -> Empty
        rb.dequeue().unwrap();
        rb.dequeue().unwrap();
        assert_eq!(rb.state, RingBufferState::Empty);
    }

    #[test]
    fn test_available() {
        let mut rb = RingBuffer::new(5);
        rb.enqueue(1);
        rb.enqueue(2);
        assert_eq!(rb.available_read(), 2);
        assert_eq!(rb.available_write(), 3);

        let byte = rb.dequeue().unwrap();
        assert_eq!(byte, 1);
        assert_eq!(rb.available_read(), 1);
        assert_eq!(rb.available_write(), 4);
    }

    #[test]
    fn test_clear() {
        let mut rb = RingBuffer::new(4);

        // Test empty buffer
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.available_read(), 0);
        assert_eq!(rb.available_write(), 4);

        // Test after writes
        rb.write(&[1, 2, 3]);
        assert_eq!(rb.available_read(), 3);
        rb.clear();
        assert!(rb.is_empty());
        assert_eq!(rb.available_write(), 4);

        // Test after wrap-around
        rb.write(&[1, 2, 3, 4]); // Fill buffer
        rb.dequeue(); // head = 1
        rb.enqueue(5); // tail wraps to 0
        rb.clear();
        assert_eq!(rb.head, 0);
        assert_eq!(rb.tail, 0);
        assert_eq!(rb.available_write(), 4);

        // Verify post-clear functionality
        rb.write(&[6, 7]);
        let mut buf = [0u8; 2];
        assert_eq!(rb.read(&mut buf), 2);
        assert_eq!(buf, [6, 7]);
    }

    #[test]
    fn test_clear_edge_cases() {
        let mut rb = RingBuffer::new(3);

        // Clear full buffer
        rb.write(&[1, 2, 3]);
        assert!(rb.is_full());
        rb.clear();
        assert!(rb.is_empty());
        rb.write(&[4, 5, 6]);
        assert_eq!(rb.drain(), vec![4, 5, 6]);

        // Clear partially read buffer
        rb.write(&[7, 8, 9]);
        rb.dequeue(); // head = 1
        rb.dequeue(); // head = 2
        rb.clear();
        rb.write(&[10]);
        assert_eq!(rb.dequeue().unwrap(), 10);
    }
}
