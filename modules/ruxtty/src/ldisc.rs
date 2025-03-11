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

use crate::termios::{Termios, CC_C_CHAR};
use alloc::{format, sync::Arc};
use axerrno::AxResult;
use axio::PollState;
use axlog::ax_print;
use ringbuffer::RingBuffer;
use spinlock::SpinNoIrq;

const BUFFER_CAPACITY: usize = 4096;

/// Line discipline managing terminal input processing
///
/// Handles raw input buffering, line editing, and terminal control settings
pub struct Ldisc {
    /// Buffer for current line being edited
    current_line: SpinNoIrq<RingBuffer>,
    /// Buffer for completed lines ready for reading
    read_buffer: SpinNoIrq<RingBuffer>,
    /// Terminal control settings
    termios: SpinNoIrq<Termios>,
    /// Terminal window dimensions
    winsize: SpinNoIrq<WinSize>,
}

impl Ldisc {
    /// Creates a new line discipline instance with default buffers and terminal settings
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            current_line: SpinNoIrq::new(RingBuffer::new(BUFFER_CAPACITY)),
            read_buffer: SpinNoIrq::new(RingBuffer::new(BUFFER_CAPACITY)),
            termios: SpinNoIrq::new(Termios::default()),
            winsize: SpinNoIrq::new(WinSize::default()),
        })
    }

    /// Reads bytes from terminal, dispatching to raw/canonical mode handlers
    pub fn read(&self, dst: &mut [u8]) -> AxResult<usize> {
        let termios = *self.termios.lock();
        if termios.is_raw_mode() {
            self.raw_mode_read(dst, &termios)
        } else {
            self.canonical_mode_read(dst, &termios)
        }
    }

    /// Canonical mode is enabled by setting the `ICANON` flag in terminal attributes.
    /// It provides line-oriented input processing with the following characteristics:  
    ///
    /// 1. **Line Assembly**:  
    ///    Input is buffered into lines terminated by any of these characters:  
    ///    - `NL` (newline, ASCII `\n`)  
    ///    - `EOL` (end-of-line, configurable via `VEOL`)  
    ///    - `EOL2` (secondary EOL, configurable via `VEOL2` if `IEXTEN` is set)  
    ///    - `EOF` (end-of-file, typically `Ctrl+D` via `VEOF`, but ignored if at line start)  
    ///
    ///    All terminators except `EOF` are included in the input line passed to the reading process.  
    ///
    /// 2. **Line Editing**:  
    ///    Supports in-line editing with these control characters:  
    ///    - `ERASE` (backspace, deletes previous character)  
    ///    - `KILL` (deletes entire line)  
    ///    - `WERASE` (word erase, deletes previous word if `IEXTEN` is set)  
    ///
    /// 3. **Read Behavior**:  
    ///    - A `read()` call returns only when a complete line is available.  
    ///    - If the requested byte count is smaller than the line length, partial data is returned; subsequent reads fetch remaining bytes.  
    ///    - (unimplemented) If interrupted by a signal without `SA_RESTART` flag, `read()` terminates early (partial read or `EINTR` error).  
    fn canonical_mode_read(&self, dst: &mut [u8], termios: &Termios) -> AxResult<usize> {
        let mut buf = self.read_buffer.lock();
        let available_read = buf.available_read();
        if available_read == 0 {
            return Err(axio::Error::WouldBlock);
        }
        let max_read_len = available_read.min(dst.len());
        let mut read_len = 0;
        while read_len < max_read_len {
            let Some(next_char) = buf.dequeue() else {
                break;
            };

            if is_line_terminator(next_char, termios) {
                if !is_eof(next_char, termios) {
                    dst[read_len] = next_char;
                    read_len += 1;
                }
                break; // Stop at line terminator
            }

            dst[read_len] = next_char;
            read_len += 1;
        }
        Ok(read_len)
    }

    /// Non-canonical mode(raw mode, enabled by unsetting `ICANON`) allows applications like `vi` and `less`
    /// to read input immediately without line termination.
    ///
    /// In this mode:  
    /// - Input bypasses line assembly and becomes visible to applications byte-by-byte.  
    /// - Special editing characters (e.g., `ERASE`, `KILL`) are disabled.  
    /// - Read completion is controlled by `VMIN` (minimum bytes) and `VTIME` (timeout) in `termios.c_cc`.  
    ///
    /// ### **VMIN and VTIME Behavior**  
    /// The interaction of `VMIN` and `VTIME` defines four distinct read modes:  
    ///
    /// #### **1. `VMIN = 0`, `VTIME = 0` (Non-Blocking Poll)**  
    /// - **Behavior**:  
    ///   - Returns immediately with available bytes (up to requested count) or `0` if no data.  
    /// - **Use Case**:  
    ///   - Non-blocking input checks (similar to `O_NONBLOCK` flag, but returns `0` instead of `EAGAIN`).  
    ///
    /// #### **2. `VMIN > 0`, `VTIME = 0` (Blocking Read)**  
    /// - **Behavior**:  
    ///   - Blocks indefinitely until at least `min(VMIN, requested_bytes)` are read.  
    /// - **Use Case**:  
    ///   - Efficient single-byte input (e.g., `less` sets `VMIN=1` to wait for keystrokes without CPU polling).  
    ///
    /// #### **3. `VMIN = 0`, `VTIME > 0` (Timeout-Based Read)**  (TODO)
    /// - **Behavior**:  
    ///   - Starts a timer (`VTIME × 0.1 seconds`) on `read()` call.  
    ///   - Returns immediately if ≥1 byte arrives or timer expires (returns `0` on timeout).  
    /// - **Use Case**:  
    ///   - Handling serial devices (e.g., modems) to avoid indefinite hangs.  
    ///
    /// #### **4. `VMIN > 0`, `VTIME > 0` (Inter-Byte Timeout)** (TODO)
    /// - **Behavior**:  
    ///   - After receiving the first byte, a timer resets for each subsequent byte.  
    ///   - Returns when:  
    ///     - `min(VMIN, requested_bytes)` are read, **OR**  
    ///     - Inter-byte gap exceeds `VTIME × 0.1 seconds` (returns available bytes ≥1).  
    /// - **Use Case**:  
    ///   - Detecting escape sequences (e.g., terminal arrow keys generating multi-byte sequences like `←` → `\x1B[D`).  
    ///   - Applications like `vi` use short timeouts (e.g., 0.2s) to distinguish keystrokes from manual input.  
    fn raw_mode_read(&self, dst: &mut [u8], termios: &Termios) -> AxResult<usize> {
        let vmin = *termios.special_char(CC_C_CHAR::VMIN);
        let vtime = *termios.special_char(CC_C_CHAR::VTIME);
        let read_len = {
            if vmin == 0 && vtime == 0 {
                self.polling_read(dst)
            } else if vmin > 0 && vtime == 0 {
                self.blocking_read(dst, vmin)?
            } else if vmin == 0 && vtime > 0 {
                todo!()
            } else if vmin > 0 && vtime > 0 {
                todo!()
            } else {
                unreachable!()
            }
        };
        Ok(read_len)
    }

    /// Used in non-canonical mode read
    ///
    /// Returns immediately with available bytes (up to requested count) or `0` if no data
    fn polling_read(&self, dst: &mut [u8]) -> usize {
        let mut buf = self.read_buffer.lock();
        let max_read_len = buf.available_read().min(dst.len());
        buf.read(&mut dst[..max_read_len])
    }

    /// Used in non-canonical mode read
    ///
    /// Blocks indefinitely until at least `min(VMIN, requested_bytes)` are read.
    fn blocking_read(&self, dst: &mut [u8], vmin: u8) -> AxResult<usize> {
        let mut buf = self.read_buffer.lock();
        let buffer_len = buf.available_read();
        if buffer_len >= dst.len() {
            return Ok(buf.read(dst));
        }
        if buffer_len < vmin as usize {
            return Err(axio::Error::WouldBlock);
        }
        Ok(buf.read(&mut dst[..buffer_len]))
    }

    /// Processes an input character through terminal line discipline
    ///
    /// Applies termios settings for character conversion, echo handling, and
    /// buffering in either raw or canonical mode.
    pub fn push_char<F: FnMut(&str)>(&self, mut ch: u8, mut echo: F) {
        let termios = self.termios.lock();

        // Convert CR to LF if ICRNL is enabled
        if termios.contain_icrnl() && ch == b'\r' {
            ch = b'\n';
        }

        // Echo handling
        if termios.contain_echo() {
            match ch {
                b'\n' => echo("\n"),   // Standard newline echo
                b'\r' => echo("\r\n"), // Carriage return expands to CR+LF
                ch if ch == *termios.special_char(CC_C_CHAR::VERASE) => {
                    // Visual backspace sequence:
                    // 1. `\x08` (Backspace) moves cursor left
                    // 2. ` ` (Space) overwrites character
                    // 3. `\x08` moves cursor left again to final position
                    // This achieves character erasure in terminal display
                    echo(core::str::from_utf8(b"\x08 \x08").unwrap());
                }
                ch if is_printable_char(ch) => {
                    // Direct echo for printable
                    ax_print!("{}", char::from(ch));
                }
                ch if is_ctrl_char(ch) && termios.contain_echo_ctl() => {
                    // Convert control character (0x01-0x1F) to ^X notation:
                    // 0x01 → 1 + 'A' - 1 = 65 → 'A' → "^A"
                    let ctrl_char = format!("^{}", char::from_u32((ch + b'A' - 1) as u32).unwrap());
                    echo(&ctrl_char);
                }
                _ => {}
            }
        }
        // Raw mode
        if termios.is_raw_mode() {
            self.read_buffer.lock().force_enqueue(ch);
            return;
        }

        // Canonical mode (line-editing)
        if ch == *termios.special_char(CC_C_CHAR::VKILL) {
            // Erase current line
            self.current_line.lock().clear();
        }

        if ch == *termios.special_char(CC_C_CHAR::VERASE) {
            // Type backspace
            let mut current_line = self.current_line.lock();
            if !current_line.is_empty() {
                current_line.dequeue();
            }
        }

        if is_line_terminator(ch, &termios) {
            // If a new line is met, all bytes in current_line will be moved to read_buffer
            let mut current_line = self.current_line.lock();
            current_line.force_enqueue(ch);
            let current_line_chars = current_line.drain();
            for char in current_line_chars {
                self.read_buffer.lock().force_enqueue(char);
            }
        }

        if is_printable_char(ch) {
            self.current_line.lock().enqueue(ch);
        }
    }

    pub fn poll(&self) -> PollState {
        let readable = !self.read_buffer.lock().is_empty();
        PollState {
            readable,
            writable: true,
            pollhup: false,
        }
    }

    pub fn termios(&self) -> Termios {
        *self.termios.lock()
    }

    pub fn set_termios(&self, termios: &Termios) {
        *self.termios.lock() = *termios
    }

    pub fn winsize(&self) -> WinSize {
        *self.winsize.lock()
    }

    pub fn set_winsize(&self, winsize: &WinSize) {
        *self.winsize.lock() = *winsize
    }

    pub fn clear_input(&self) {
        self.current_line.lock().clear();
        self.read_buffer.lock().clear();
    }
}

/// Terminal window size information (rows/columns)
///
/// Follows POSIX `struct winsize` convention, pixel fields unused in most TTY implementations
#[derive(Debug, Clone, Copy, Default)]
#[repr(C)]
pub struct WinSize {
    /// Number of terminal rows
    ws_row: u16,
    /// Number of terminal columns
    ws_col: u16,
    /// Horizontal size in pixels
    ws_xpixel: u16,
    /// Vertical size in pixels
    ws_ypixel: u16,
}

/// printable characters​ refer to characters that can be visually displayed in terminals
///
/// ASCII Printable Characters: Ranging from decimal 32 (space) to 126 (the ~ symbol),
/// covering letters (uppercase and lowercase), digits, punctuation marks (e.g., !@#$%), and spaces.
fn is_printable_char(ch: u8) -> bool {
    (0x20..0x7f).contains(&ch)
}

/// Checks if the character is a control character (0x00-0x1F), excluding \r and \n
fn is_ctrl_char(ch: u8) -> bool {
    if ch == b'\r' || ch == b'\n' {
        return false;
    }
    (0..0x20).contains(&ch)
}

/// Checks if the character matches the EOF control character (VEOF)
fn is_eof(ch: u8, termios: &Termios) -> bool {
    ch == *termios.special_char(CC_C_CHAR::VEOF)
}

/// Checks if the character is a line terminator (\n, VEOF, VEOL, or VEOL2 with IEXTEN)
fn is_line_terminator(ch: u8, termios: &Termios) -> bool {
    if ch == b'\n'
        || ch == *termios.special_char(CC_C_CHAR::VEOF)
        || ch == *termios.special_char(CC_C_CHAR::VEOL)
    {
        return true;
    }
    if termios.contain_iexten() && ch == *termios.special_char(CC_C_CHAR::VEOL2) {
        return true;
    }
    false
}
