#![allow(clippy::upper_case_acronyms)]
#![allow(dead_code)]
use num_enum::FromPrimitive;
#[derive(Debug, FromPrimitive, Clone, Copy, Eq, PartialEq)]
#[repr(u32)]
/// Tty IO control command
pub enum IoctlCommand {
    /// Default variant to catch invalid/unknown ioctl commands
    #[num_enum(default)]
    InvalidCommand,

    // ==============================================
    // Terminal control commands (termios structure)
    // ==============================================
    /// TCGETS: Terminal Control GET Settings  
    ///
    /// Gets current terminal settings using termios structure
    TCGETS = 0x5401,

    /// TCSETS: Terminal Control SET Settings
    ///
    /// Sets terminal settings immediately using termios structure
    TCSETS = 0x5402,

    /// TCSETSW: Terminal Control SET Settings and Wait
    ///
    /// Sets terminal settings after draining output buffer
    TCSETSW = 0x5403,

    /// TCSETSF: Terminal Control SET Settings and Flush
    ///
    /// Sets terminal settings after flushing input/output buffers
    TCSETSF = 0x5404,

    // ==============================================
    // Terminal control commands (termio structure - legacy BSD)
    // ==============================================
    /// TCGETA: Terminal Control GET Attributes
    ///
    /// Gets current terminal settings using legacy termio structure
    TCGETA = 0x5405,

    /// TCSETA: Terminal Control SET Attributes
    ///
    /// Sets terminal settings immediately using termio structure
    TCSETA = 0x5406,

    /// TCSETAW: Terminal Control SET Attributes and Wait
    ///
    /// Sets termio settings after draining output buffer
    TCSETAW = 0x5407,

    /// TCSETAF: Terminal Control SET Attributes and Flush
    ///
    /// Sets termio settings after flushing input/output buffers
    TCSETAF = 0x5408,

    // ==============================================
    // Special control commands
    // ==============================================
    /// TCSBRK: Terminal Control Send BReaK
    ///
    /// Sends a break sequence (stream of zero bits) for 0.25-0.5 seconds
    TCSBRK = 0x5409,

    /// TIOCSCTTY: Terminal IOCtl Set Controlling TTY
    ///
    /// Makes the given terminal the controlling terminal of the calling process
    TIOCSCTTY = 0x540E,

    /// TIOCGPGRP: Terminal IOCtl Get Process GRouP
    ///
    /// Gets foreground process group ID associated with terminal
    TIOCGPGRP = 0x540F,

    /// TIOCSPGRP: Terminal IOCtl Set Process GRouP
    ///
    /// Sets foreground process group ID associated with terminal
    TIOCSPGRP = 0x5410,

    /// TIOCGWINSZ: Terminal IOCtl Get WINdow SiZe
    ///
    /// Gets terminal window size (rows/columns)
    TIOCGWINSZ = 0x5413,

    /// TIOCSWINSZ: Terminal IOCtl Set WINdow SiZe
    ///
    /// Sets terminal window size (rows/columns)
    TIOCSWINSZ = 0x5414,

    /// TIOCNOTTY: Terminal IOCtl No TTY
    ///
    /// Disassociates from controlling terminal (used by daemons/sshd)
    TIOCNOTTY = 0x5422,

    /// Lock/unlock Pty
    TIOCSPTLCK = 0x40045431,

    /// Given a file descriptor in fd that refers to a pseudoterminal master,
    /// open and return a new file descriptor that refers to the peer pseudoterminal slave device.
    TIOCGPTPEER = 0x40045441,

    /// Get Pty Number
    TIOCGPTN = 0x80045430,
}
