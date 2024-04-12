/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! functions for tty buffer.
//! Drivers should fill the buffer by functions below.
//! then the data will be passed to line discipline for processing.

/// tty buffer size.
const TTY_BUF_SIZE: usize = 4096;

/// ring buffer.
#[derive(Debug)]
struct RingBuffer {
    /// data.
    buf: [u8; TTY_BUF_SIZE],

    /// the first element or empty slot if buffer is empty.
    head: usize,

    /// the first empty slot.
    tail: usize,

    /// number of elements.
    len: usize,
}

/// tty buffer.
/// TODO: use flip buffer.
#[derive(Debug)]
pub struct TtyBuffer {
    /// use ring buffer to save chars.
    buffer: spinlock::SpinNoIrq<RingBuffer>,
}

impl TtyBuffer {
    pub fn new() -> Self {
        Self {
            buffer: spinlock::SpinNoIrq::new(RingBuffer {
                buf: [0u8; TTY_BUF_SIZE],
                head: 0,
                tail: 0,
                len: 0,
            }),
        }
    }

    /// get `index`th element without changing buffer.
    pub fn see(&self, index: usize) -> u8 {
        let buf = self.buffer.lock();
        if index < buf.len {
            buf.buf[(index + buf.head) % TTY_BUF_SIZE]
        } else {
            0
        }
    }

    /// push a char to tail.
    pub fn push(&self, ch: u8) {
        let mut buf = self.buffer.lock();
        if buf.len != TTY_BUF_SIZE {
            buf.len += 1;
            let idx = buf.tail;
            buf.buf[idx] = ch;
            buf.tail = (buf.tail + 1) % TTY_BUF_SIZE;
        }
    }

    /// delete and return the heading char.
    pub fn pop(&self) -> u8 {
        self.delete(0)
    }

    /// insert `ch` to `index`th position.
    pub fn insert(&self, ch: u8, index: usize) {
        let mut buf = self.buffer.lock();
        // if not full and index is right
        if buf.len != TTY_BUF_SIZE && index <= buf.len {
            // shift buffer[index..move_len+index] one slot right.
            let move_len = buf.len - index;
            let mut i = buf.tail;
            for _ in 0..move_len {
                i -= 1;
                buf.buf[(i + 1) % TTY_BUF_SIZE] = buf.buf[i % TTY_BUF_SIZE];
            }
            // insert
            let idx = (buf.head + index) % TTY_BUF_SIZE;
            buf.buf[idx] = ch;
            buf.len += 1;
            buf.tail = (buf.tail + 1) % TTY_BUF_SIZE;
        }
    }

    /// delete and return the `index`th element.
    pub fn delete(&self, index: usize) -> u8 {
        let mut buf = self.buffer.lock();
        // if not empty and index is right
        if buf.len != 0 && index < buf.len {
            let move_len = buf.len - index;
            let mut i = index + buf.head;

            // save retval
            let ret = buf.buf[i % TTY_BUF_SIZE];

            // copy move_len elements from buffer[index+head] to buffer[index+head-1];
            for _ in 0..move_len {
                buf.buf[i % TTY_BUF_SIZE] = buf.buf[(i + 1) % TTY_BUF_SIZE];
                i += 1;
            }

            // len -= 1
            buf.len -= 1;
            buf.tail -= 1;
            ret
        } else {
            0
        }
    }

    /// get current length of buffer.
    pub fn len(&self) -> usize {
        self.buffer.lock().len
    }
}

/// a buffer for echo of line discipline.
/// additionally saving the cursor position.
#[derive(Debug)]
pub struct EchoBuffer {
    /// chars buffer.
    pub buffer: TtyBuffer,

    /// current column of cursor.
    pub col: usize,
}

impl EchoBuffer {
    pub fn new() -> Self {
        Self {
            buffer: TtyBuffer::new(),
            col: 0,
        }
    }
}
