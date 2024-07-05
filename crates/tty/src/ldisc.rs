/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! TTY line discipline process all incoming and outgoing chars from/to a tty device.
//! the currently implemented line discipline is N_TTY.
//! line disciplines are registered when a device is registered.

use alloc::sync::Arc;
use spinlock::SpinNoIrq;

use crate::{
    buffer::{EchoBuffer, TtyBuffer},
    tty::TtyStruct,
};

/// tty line discipline.
#[derive(Debug)]
pub struct TtyLdisc {
    /// chars that can be read by kernel.
    read_buf: TtyBuffer,

    /// chars being echoed on the screen.
    echo_buf: SpinNoIrq<EchoBuffer>,

    /// chars from driver, and not yet been processed.
    rec_buf: TtyBuffer,
}

/// implement N_TTY.
impl TtyLdisc {
    pub fn new() -> Self {
        Self {
            read_buf: TtyBuffer::new(),
            echo_buf: SpinNoIrq::new(EchoBuffer::new()),
            rec_buf: TtyBuffer::new(),
        }
    }

    /// kernel reads data.
    pub fn read(&self, buf: &mut [u8]) -> usize {
        let read_buf = &self.read_buf;

        // len of this reading
        let len = buf.len().min(read_buf.len());

        // return if nothing can be read
        if len == 0 {
            return 0;
        }

        // copy data from read_buf to `buf`
        for ch in buf.iter_mut().take(len) {
            *ch = read_buf.pop();
        }

        len
    }

    /// driver sends data from device for processing and echoing.
    /// running in irq.
    pub fn receive_buf(&self, tty: Arc<TtyStruct>, buf: &[u8]) {
        use crate::constant::*;

        let rec_buf = &self.rec_buf;

        // save data to receive buffer
        for ch in buf {
            rec_buf.push(*ch);
        }

        // process chars in receive buffer
        while rec_buf.len() > 0 {
            let ch = rec_buf.see(0);

            // if char may be arrow char
            if ch == ARROW_PREFIX[0] {
                // no enough len, just break, waitting for next time
                if rec_buf.len() < 3 {
                    break;
                }

                // enough len, but not a arrow char, just ignore
                if rec_buf.see(1) != ARROW_PREFIX[1] {
                    rec_buf.pop();
                    rec_buf.pop();
                    break;
                }

                // it is an arrow char, get it
                rec_buf.pop();
                rec_buf.pop();
                let ch = rec_buf.pop();

                // deal with arrow char
                match ch {
                    LEFT => {
                        let mut lock = self.echo_buf.lock();
                        // if can go left
                        if lock.col > 0 {
                            self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], ch]);
                            lock.col -= 1;
                        }
                    }
                    RIGHT => {
                        let mut lock = self.echo_buf.lock();
                        // if can go right
                        if lock.col < lock.buffer.len() {
                            self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], ch]);
                            lock.col += 1;
                        }
                    }
                    _ => {
                        // it is UP/DOWN, just ignore
                    }
                }
            // not a arrow char, handle it as a normal char
            } else {
                let ch = rec_buf.pop();
                match ch {
                    CR | LF => {
                        // always '\n'
                        let ch = LF;

                        // echo
                        self.write(tty.clone(), &[ch]);

                        // push this char to echo buffer
                        let mut lock = self.echo_buf.lock();
                        lock.buffer.push(ch);

                        // copy echo buffer to read buffer
                        // FIXME: currently will push all data to read_buf
                        let len = lock.buffer.len();
                        for _ in 0..len {
                            self.read_buf.push(lock.buffer.pop());
                        }

                        // echo buffer's column is set to 0
                        lock.col = 0;
                    }
                    BS | DEL => {
                        let mut lock = self.echo_buf.lock();
                        let col = lock.col;
                        let len = lock.buffer.len();
                        // if can delete
                        if col > 0 {
                            // perform a backspace
                            self.write(tty.clone(), &[BS, SPACE, BS]);

                            // if cursor is not on the rightmost
                            if col != len {
                                for i in col..len {
                                    let ch = lock.buffer.see(i);
                                    self.write(tty.clone(), &[ch]);
                                }
                                self.write(tty.clone(), &[SPACE]);
                                for _ in 0..(len - col + 1) {
                                    self.write(
                                        tty.clone(),
                                        &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT],
                                    );
                                }
                            }

                            // modify echo buffer
                            lock.buffer.delete(col - 1);
                            lock.col -= 1;
                        }
                    }
                    _ => {
                        // process normal chars.
                        let mut echo_buf = self.echo_buf.lock();
                        let col = echo_buf.col;
                        let len = echo_buf.buffer.len();

                        // echo
                        self.write(tty.clone(), &[ch]);

                        // if cursor is not on the rightmost
                        if col != len {
                            for i in col..len {
                                self.write(tty.clone(), &[echo_buf.buffer.see(i)]);
                            }
                            for _ in 0..(len - col) {
                                self.write(tty.clone(), &[ARROW_PREFIX[0], ARROW_PREFIX[1], LEFT]);
                            }
                        }

                        // modify echo buffer
                        echo_buf.buffer.insert(ch, col);
                        echo_buf.col += 1;
                    }
                }
            }
        }
    }

    /// kernel writes data to device.
    pub fn write(&self, tty: Arc<TtyStruct>, buf: &[u8]) -> usize {
        let mut len = 0;
        let driver = tty.driver();
        for ch in buf {
            len += 1;
            // call driver's method
            (driver.ops.putchar)(*ch);
        }
        len
    }
}

impl Default for TtyLdisc {
    fn default() -> Self {
        Self::new()
    }
}
