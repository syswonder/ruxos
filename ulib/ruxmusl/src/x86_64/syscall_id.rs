use num_enum::TryFromPrimitive;

// TODO: syscall id are architecture-dependent
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
pub enum SyscallId {
    INVALID = 999,

    READ = 0,
    WRITE = 1,

    #[cfg(feature = "fs")]
    OPEN = 2,

    #[cfg(feature = "fd")]
    CLOSE = 3,

    #[cfg(feature = "fs")]
    STAT = 4,

    #[cfg(feature = "fs")]
    FSTAT = 5,

    #[cfg(feature = "fs")]
    LSTAT = 6,

    #[cfg(feature = "poll")]
    POLL = 7,

    #[cfg(feature = "fs")]
    LSEEK = 8,

    #[cfg(feature = "alloc")]
    MMAP = 9,

    #[cfg(feature = "alloc")]
    MPROTECT = 10,

    #[cfg(feature = "alloc")]
    MUNMAP = 11,

    #[cfg(feature = "signal")]
    RT_SIGACTION = 13,

    #[cfg(feature = "signal")]
    RT_SIGPROCMASK = 14,

    #[cfg(feature = "fd")]
    IOCTL = 16,

    #[cfg(feature = "fs")]
    PREAD64 = 17,

    #[cfg(feature = "fs")]
    PWRITE64 = 18,

    #[cfg(feature = "fd")]
    READV = 19,

    #[cfg(feature = "fd")]
    WRITEV = 20,

    #[cfg(feature = "pipe")]
    PIPE = 22,

    #[cfg(feature = "select")]
    SELECT = 23,

    SCHED_YIELD = 24,

    #[cfg(feature = "alloc")]
    MREMAP = 25,

    #[cfg(feature = "alloc")]
    MSYNC = 26,

    #[cfg(feature = "alloc")]
    MADVISE = 28,

    #[cfg(feature = "fd")]
    DUP = 32,

    #[cfg(feature = "fd")]
    DUP2 = 33,

    NANO_SLEEP = 35,

    #[cfg(feature = "multitask")]
    GETPID = 39,

    #[cfg(feature = "net")]
    SOCKET = 41,

    #[cfg(feature = "net")]
    CONNECT = 42,

    #[cfg(feature = "net")]
    ACCEPT = 43,

    #[cfg(feature = "net")]
    SENDTO = 44,

    #[cfg(feature = "net")]
    RECVFROM = 45,

    #[cfg(feature = "net")]
    SENDMSG = 46,

    #[cfg(feature = "net")]
    SHUTDOWN = 48,

    #[cfg(feature = "net")]
    BIND = 49,

    #[cfg(feature = "net")]
    LISTEN = 50,

    #[cfg(feature = "net")]
    GETSOCKNAME = 51,

    #[cfg(feature = "net")]
    GETPEERNAME = 52,

    #[cfg(feature = "net")]
    SETSOCKOPT = 54,

    // TODO: check clone
    #[cfg(feature = "multitask")]
    CLONE = 56,

    #[cfg(feature = "fs")]
    EXECVE = 59,

    EXIT = 60,

    #[cfg(feature = "signal")]
    KILL = 62,

    UNAME = 63,

    #[cfg(feature = "fd")]
    FCNTL = 72,

    #[cfg(feature = "fs")]
    FSYNC = 74,

    #[cfg(feature = "fs")]
    FDATASYNC = 75,

    #[cfg(feature = "fs")]
    GETDENTS = 78,

    #[cfg(feature = "fs")]
    GETCWD = 79,

    #[cfg(feature = "fs")]
    CHDIR = 80,

    #[cfg(feature = "fs")]
    RENAME = 82,

    #[cfg(feature = "fs")]
    MKDIR = 83,

    #[cfg(feature = "fs")]
    RMDIR = 84,

    #[cfg(feature = "fs")]
    UNLINK = 87,

    #[cfg(feature = "fs")]
    READLINK = 89,

    UMASK = 95,

    GETTIMEOFDAY = 96,

    GETRLIMIT = 97,

    SYSINFO = 99,

    TIMES = 100,

    GETUID = 102,

    GETGID = 104,

    SETUID = 105,

    SETGID = 106,

    GETPPID = 110,

    GETPGID = 121,

    CAPGET = 125,

    #[cfg(feature = "signal")]
    SIGALTSTACK = 131,

    PRCTL = 157,

    ARCH_PRCTL = 158,

    #[cfg(feature = "multitask")]
    GETTID = 186,

    #[cfg(feature = "multitask")]
    FUTEX = 202,

    #[cfg(feature = "epoll")]
    EPOLL_CREATE = 213,

    #[cfg(feature = "fs")]
    GETDENTS64 = 217,

    #[cfg(feature = "multitask")]
    SET_TID_ADDRESS = 218,

    CLOCK_SETTIME = 227,

    CLOCK_GETTIME = 228,

    #[cfg(feature = "epoll")]
    EPOLL_WAIT = 232,

    #[cfg(feature = "epoll")]
    EPOLL_CTL = 233,

    #[cfg(feature = "fs")]
    OPENAT = 257,

    #[cfg(feature = "fs")]
    MKDIRAT = 258,

    #[cfg(feature = "fs")]
    NEWFSTATAT = 262,

    #[cfg(feature = "fs")]
    UNLINKAT = 263,

    #[cfg(feature = "fs")]
    RENAMEAT = 264,

    #[cfg(feature = "fs")]
    READLINKAT = 267,

    #[cfg(feature = "fs")]
    FACCESSAT = 269,

    #[cfg(feature = "select")]
    PSELECT6 = 270,

    #[cfg(feature = "poll")]
    PPOLL = 271,

    #[cfg(feature = "epoll")]
    EPOLL_PWAIT = 281,

    #[cfg(feature = "epoll")]
    EPOLL_CREATE1 = 291,

    #[cfg(feature = "fd")]
    DUP3 = 292,

    #[cfg(feature = "pipe")]
    PIPE2 = 293,

    #[cfg(feature = "fs")]
    PREADV = 295,

    PRLIMIT64 = 302,

    GETRANDOM = 318,
}
