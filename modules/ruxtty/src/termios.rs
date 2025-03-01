#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
use bitflags::bitflags;
const NCCS: usize = 19;
type CcT = u8;

/// Represents terminal I/O settings and control characters.
///
/// This structure mirrors the behavior of the POSIX `termios` structure,
/// providing configuration for serial communication, line discipline, and
/// terminal control. It is typically used with `tcgetattr` and `tcsetattr`
/// for configuring TTY devices.
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Termios {
    /// Input mode flags
    c_iflags: C_IFLAGS,
    /// Output mode flags
    c_oflags: C_OFLAGS,
    /// Control mode flags (e.g., baud rate)
    c_cflags: C_CFLAGS,
    /// Local mode flags (e.g., echo, canonical mode).  
    c_lflags: C_LFLAGS,
    /// Line discipline type
    c_line: CcT,
    /// Array of control characters
    c_cc: [CcT; NCCS],
}

impl Default for Termios {
    fn default() -> Self {
        Self {
            c_iflags: C_IFLAGS::default(),
            c_oflags: C_OFLAGS::default(),
            c_cflags: C_CFLAGS::default(),
            c_lflags: C_LFLAGS::default(),
            c_line: 0,
            c_cc: [
                3,   // VINTR Ctrl-C
                28,  // VQUIT
                127, // VERASE
                21,  // VKILL
                4,   // VEOF Ctrl-D
                0,   // VTIME
                1,   // VMIN
                0,   // VSWTC
                17,  // VSTART
                19,  // VSTOP
                26,  // VSUSP Ctrl-Z
                255, // VEOL
                18,  // VREPAINT
                15,  // VDISCARD
                23,  // VWERASE
                22,  // VLNEXT
                255, // VEOL2
                0, 0,
            ],
        }
    }
}

impl Termios {
    /// Gets the value of a specific control character (e.g. VINTR, VERASE)
    pub fn special_char(&self, cc_c_char: CC_C_CHAR) -> &CcT {
        &self.c_cc[cc_c_char as usize]
    }

    /// Checks if terminal is in raw mode (non-canonical input processing)
    pub fn is_raw_mode(&self) -> bool {
        !self.c_lflags.contains(C_LFLAGS::ICANON)
    }

    /// Checks if carriage return to newline conversion is enabled (\r → \n)
    pub fn contain_icrnl(&self) -> bool {
        self.c_iflags.contains(C_IFLAGS::ICRNL)
    }

    /// Checks if signal-generating characters (e.g. Ctrl+C) are enabled
    pub fn contain_isig(&self) -> bool {
        self.c_lflags.contains(C_LFLAGS::ISIG)
    }

    /// Checks if input character echoing is enabled
    pub fn contain_echo(&self) -> bool {
        self.c_lflags.contains(C_LFLAGS::ECHO)
    }

    /// Checks if control characters are echoed as ^X
    pub fn contain_echo_ctl(&self) -> bool {
        self.c_lflags.contains(C_LFLAGS::ECHOCTL)
    }

    /// Checks if extended input character processing is enabled
    pub fn contain_iexten(&self) -> bool {
        self.c_lflags.contains(C_LFLAGS::IEXTEN)
    }
}
bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    /// Input mode flags
    pub struct C_IFLAGS: u32 {
        // ==============================================
        // Break condition handling
        // ==============================================

        /// IGNBRK: IGNore BReaK condition
        /// Ignore break conditions (BREAK treated as valid NULL byte)
        const IGNBRK  = 0x001;

        /// BRKINT: BReaK INTerrupt
        /// Generate SIGINT on break condition (when not ignored by IGNBRK)
        const BRKINT  = 0x002;

        // ==============================================
        // Parity/error handling
        // ==============================================

        /// IGNPAR: IGNore PARity errors
        /// Ignore bytes with framing/parity errors (keep invalid bytes)
        const IGNPAR  = 0x004;

        /// PARMRK: PARity MarK
        /// Insert error marker bytes (0xFF + 0x00) before invalid bytes
        const PARMRK  = 0x008;

        /// INPCK: INput Parity ChecK
        /// Enable parity checking for incoming bytes
        const INPCK   = 0x010;

        // ==============================================
        // Character processing
        // ==============================================

        /// ISTRIP: STRIP high bit
        /// Strip 8th bit of input characters (force 7-bit processing)
        const ISTRIP  = 0x020;

        /// INLCR: INput NL to CR
        /// Map received newline (NL, 0x0A) to carriage return (CR, 0x0D)
        const INLCR   = 0x040;

        /// IGNCR: IGNore CR
        /// Ignore received carriage return (CR, 0x0D) characters
        const IGNCR   = 0x080;

        /// ICRNL: Input CR to NL
        /// Map received carriage return (CR, 0x0D) to newline (NL, 0x0A)
        const ICRNL   = 0x100;

        /// IUCLC: Input UpperCase to LowerCase
        /// Map uppercase letters (A-Z) to lowercase (a-z) on input
        const IUCLC   = 0x0200;

        // ==============================================
        // Flow control
        // ==============================================

        /// IXON: Enable XON/XOFF output control
        /// Enable software flow control for output (Ctrl-Q/Ctrl-S)
        const IXON    = 0x0400;

        /// IXOFF: Enable XON/XOFF input control
        /// Enable software flow control for input (Ctrl-Q/Ctrl-S)
        const IXOFF   = 0x1000;

        /// IXANY: Enable any character to restart output
        /// Allow any character (not just XON) to resume paused output
        const IXANY   = 0x800;

        // ==============================================
        // Special behaviors
        // ==============================================

        /// IMAXBEL: Input MAX buffer BEL
        /// Ring terminal bell when input buffer is full
        const IMAXBEL = 0x2000;

        /// IUTF8: Input UTF-8
        /// Enable UTF-8 input processing (required for canonical mode)
        const IUTF8   = 0x4000;
    }
}
impl Default for C_IFLAGS {
    fn default() -> Self {
        C_IFLAGS::ICRNL | C_IFLAGS::IXON
    }
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    /// Output mode flags
    pub struct C_OFLAGS: u32 {
        // ==============================================
        // Output processing control
        // ==============================================

        /// OPOST: Output POST-process
        /// Enable output post-processing (required for other transformations)
        const OPOST  = 1 << 0;

        /// ONLCR: Output NL to CR-NL
        /// Map newline (NL, 0x0A) to CR-NL sequence (0x0D 0x0A)
        const ONLCR  = 1 << 2;

        /// OCRNL: Output CR to NL
        /// Map carriage return (CR, 0x0D) to newline (NL, 0x0A)
        const OCRNL  = 1 << 3;

        /// ONOCR: Output No CR at column 0
        /// Discard CR characters when at start of line (column 0)
        const ONOCR  = 1 << 4;

        /// ONLRET: Output NL performs RETurn
        /// Newline (NL) moves cursor to column 0 (carriage return behavior)
        const ONLRET = 1 << 5;

        // ==============================================
        // Character case conversion
        // ==============================================

        /// OLCUC: Output LowerCase UpperCase
        /// Convert lowercase letters (a-z) to uppercase (A-Z) on output
        const OLCUC  = 1 << 1;

        // ==============================================
        // Fill/delay handling (legacy systems)
        // ==============================================

        /// OFILL: Output FILL characters
        /// Send fill characters for timing delays (obsolete on modern systems)
        const OFILL  = 1 << 6;

        /// OFDEL: Output DEL as fill character
        /// Use DEL (0x7F) instead of NUL (0x00) for fill (requires OFILL)
        const OFDEL  = 1 << 7;
    }
}

impl Default for C_OFLAGS {
    fn default() -> Self {
        C_OFLAGS::OPOST | C_OFLAGS::ONLCR
    }
}

/// Control mode flags
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct C_CFLAGS(u32);

impl Default for C_CFLAGS {
    /// Creates default c_cflags configuration:
    /// - Baud rate: 38400 (B38400)
    /// - Data bits: 8 (CS8)
    /// - Enable receiver (CREAD)
    fn default() -> Self {
        /// Enable receiver flag (termios CREAD)
        const CREAD: u32 = 0x00000080;
        let cbaud = C_CFLAGS_BAUD::B38400 as u32;
        let csize = C_CFLAGS_CSIZE::CS8 as u32;
        let c_cflags = cbaud | csize | CREAD;
        Self(c_cflags)
    }
}

/// Baud rate constants for termios c_cflags field
///
/// These values correspond to the `B*` constants in Unix termios implementations.
/// Used to set serial communication speed. The special value `B0` indicates hang-up.
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
#[doc(hidden)]
pub enum C_CFLAGS_BAUD {
    /// Hang up/discard data
    B0 = 0x00000000,
    B50 = 0x00000001,
    B75 = 0x00000002,
    B110 = 0x00000003,
    B134 = 0x00000004,
    B150 = 0x00000005,
    B200 = 0x00000006,
    B300 = 0x00000007,
    B600 = 0x00000008,
    B1200 = 0x00000009,
    B1800 = 0x0000000a,
    B2400 = 0x0000000b,
    B4800 = 0x0000000c,
    B9600 = 0x0000000d,
    B19200 = 0x0000000e,
    B38400 = 0x0000000f,
}

bitflags! {
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    /// Local mode flags (e.g., echo, canonical mode).
    pub struct C_LFLAGS: u32 {
        /// Generate signals (SIGINT/SIGQUIT/SIGSTOP) for control characters.
        const ISIG    = 0x00001;
        /// Enable canonical input mode (line-by-line input with editing).
        const ICANON  = 0x00002;
        /// Obsolete flag for uppercase/lowercase input processing (use `IUTF8` instead).
        const XCASE   = 0x00004;
        /// Echo input characters back to the terminal.
        const ECHO    = 0x00008;
        /// Echo ERASE character as backspace-space-backspace sequence (visual erase).
        const ECHOE   = 0x00010;
        /// Echo KILL character (line deletion) by erasing the entire line.
        const ECHOK   = 0x00020;
        /// Echo newline (`NL`) even if `ECHO` is disabled.
        const ECHONL  = 0x00040;
        /// Disable flushing after SIGINT/SIGQUIT signals.
        const NOFLSH  = 0x00080;
        /// Send SIGTTOU when background processes write to terminal (job control).
        const TOSTOP  = 0x00100;
        /// Echo control characters as `^X` (Ctrl-X notation).
        const ECHOCTL = 0x00200;
        /// Echo erased characters visually (between `\` and `/` during erase).
        const ECHOPRT = 0x00400;
        /// Erase entire line on KILL character (enhanced `ECHOK` behavior).
        const ECHOKE  = 0x00800;
        /// Output is being flushed (set internally; do not configure manually).
        const FLUSHO  = 0x01000;
        /// Redraw pending input after next read (typeahead handling for line editing).
        const PENDIN  = 0x04000;
        /// Enable extended input character processing (non-canonical mode features).
        const IEXTEN  = 0x08000;
        /// External processing mode (used with `ICANON` for shell job control).
        const EXTPROC = 0x10000;
    }
}

impl Default for C_LFLAGS {
    fn default() -> Self {
        C_LFLAGS::ICANON
            | C_LFLAGS::ECHO
            | C_LFLAGS::ISIG
            | C_LFLAGS::ECHOE
            | C_LFLAGS::ECHOK
            | C_LFLAGS::ECHOCTL
            | C_LFLAGS::ECHOKE
            | C_LFLAGS::IEXTEN
    }
}

#[repr(u32)]
#[doc(hidden)]
#[derive(Clone, Copy)]
/// These values correspond to the `CS*` constants in Unix termios implementations.
/// Used with `CSIZE_MASK` to set/clear the number of data bits per character.
pub enum C_CFLAGS_CSIZE {
    CS5 = 0x00000000,
    CS6 = 0x00000010,
    CS7 = 0x00000020,
    CS8 = 0x00000030,
}

/* c_cc characters index*/
#[repr(u32)]
#[derive(Debug, Clone, Copy)]
/// Represents indices for terminal control characters in the `c_cc` array of [`Termios`].
///
/// These constants define positions in the control character array (`c_cc`), which configure
/// terminal behavior (e.g., signal triggers, line editing). Values correspond to the POSIX standard
pub enum CC_C_CHAR {
    /// Interrupt character (e.g., `Ctrl+C`). Sends `SIGINT` to the foreground process group.
    VINTR = 0,
    /// Quit character (e.g., `Ctrl+\`). Sends `SIGQUIT` to the foreground process group.
    VQUIT = 1,
    /// Erase character (e.g., `Backspace` or `Ctrl+?`). Deletes the preceding character in line-editing mode.
    VERASE = 2,
    /// Kill character (e.g., `Ctrl+U`). Deletes the entire line in line-editing mode.
    VKILL = 3,
    /// End-of-file character (e.g., `Ctrl+D`). Signals EOF in canonical input mode.
    VEOF = 4,
    /// Timeout value (in tenths of a second) for non-canonical reads. Used with [`VMIN`](Self::VMIN).
    VTIME = 5,
    /// Minimum number of bytes to read in non-canonical mode. Used with [`VTIME`](Self::VTIME).
    VMIN = 6,
    /// Switch process group character (obsolete; unused in modern systems).
    VSWTC = 7,
    /// Resume output character (e.g., `Ctrl+Q`). Restarts paused output (XON).
    VSTART = 8,
    /// Suspend output character (e.g., `Ctrl+S`). Pauses terminal output (XOFF).
    VSTOP = 9,
    /// Suspend character (e.g., `Ctrl+Z`). Sends `SIGTSTP` to the foreground process group.
    VSUSP = 10,
    /// End-of-line character (alternate; `\0` by default). Terminates input in canonical mode.
    VEOL = 11,
    /// Reprint character (e.g., `Ctrl+R`). Redisplays the current line in line-editing mode.
    VREPRINT = 12,
    /// Discard character (e.g., `Ctrl+O`). Toggles discarding of pending output.
    VDISCARD = 13,
    /// Word-erase character (e.g., `Ctrl+W`). Deletes the preceding word in line-editing mode.
    VWERASE = 14,
    /// Literal-next character (e.g., `Ctrl+V`). Quotes the next input character (disables special handling).
    VLNEXT = 15,
    /// Secondary end-of-line character (rarely used). Alternative to [`VEOL`](Self::VEOL).
    VEOL2 = 16,
}
