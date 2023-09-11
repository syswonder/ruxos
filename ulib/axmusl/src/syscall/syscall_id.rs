use num_enum::TryFromPrimitive;

// TODO: syscall id are architecture-dependent
#[allow(clippy::upper_case_acronyms)]
#[allow(non_camel_case_types)]
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, Copy, TryFromPrimitive)]
pub enum SyscallId {
    INVALID = 999,
    #[cfg(feature = "fd")]
    IOCTL = 29,
    #[cfg(feature = "fs")]
    OPENAT = 56,
    READ = 63,
    WRITE = 64,
    #[cfg(feature = "fd")]
    CLOSE = 57,
    // #[cfg(feature = "fs")]
    // STAT = ,
    #[cfg(feature = "fs")]
    FSTAT = 80,
    #[cfg(feature = "multitask")]
    SET_TID_ADDRESS = 96,
    // #[cfg(feature = "fs")]
    // LSTAT = ,
    #[cfg(feature = "fs")]
    LSEEK = 62,
    #[cfg(feature = "fd")]
    WRITEV = 66,
    // TODO: this should be architecture-dependent
    #[cfg(feature = "pipe")]
    PIPE2 = 59,
    // #[cfg(feature = "select")]
    // SELECT = 23,
    SCHED_YIELD = 124,
    #[cfg(feature = "fd")]
    DUP = 23,
    // #[cfg(feature = "fd")]
    // DUP2 = 33,
    NANO_SLEEP = 101,
    #[cfg(feature = "multitask")]
    GETPID = 172,
    #[cfg(feature = "net")]
    SOCKET = 198,
    #[cfg(feature = "net")]
    CONNECT = 203,
    #[cfg(feature = "net")]
    ACCEPT = 202,
    #[cfg(feature = "net")]
    SENDTO = 206,
    #[cfg(feature = "net")]
    RECVFROM = 207,
    #[cfg(feature = "net")]
    SHUTDOWN = 210,
    #[cfg(feature = "net")]
    BIND = 200,
    #[cfg(feature = "net")]
    LISTEN = 201,
    #[cfg(feature = "net")]
    GETSOCKNAME = 204,
    #[cfg(feature = "net")]
    GETPEERNAME = 205,
    EXIT = 93,
    #[cfg(feature = "fd")]
    FCNTL = 25,
    #[cfg(feature = "fs")]
    GETCWD = 17,
    // #[cfg(feature = "fs")]
    // RENAME = ,
    // #[cfg(feature = "epoll")]
    // EPOLL_CREATE = 213,
    CLOCK_GETTIME = 113,
    // TODO: epoll_wait or epoll_pwait?
    // #[cfg(feature = "epoll")]
    // EPOLL_WAIT = 232,
    #[cfg(feature = "epoll")]
    EPOLL_CTL = 21,
    #[cfg(feature = "multitask")]
    FUTEX = 98,
    #[cfg(feature = "alloc")]
    RT_SIGPROCMASK = 135,
    #[cfg(feature = "alloc")]
    MUNMAP = 215,
    #[cfg(feature = "multitask")]
    CLONE = 220,
    #[cfg(feature = "alloc")]
    MMAP = 222,
    #[cfg(feature = "alloc")]
    MPROTECT = 226,
    // #[cfg(feature = "fd")]
    // DUP3 = 292,

    // // ArceOS specific syscall, starting from 500
    // /// `send` should call `sendto`
    // #[cfg(feature = "net")]
    // SEND = 500,
    // /// `recv` should call `recvfrom`
    // #[cfg(feature = "net")]
    // RECV = 501,
    // /// This is not a syscall, but requires `dns send` in ArceOS
    // #[cfg(feature = "net")]
    // GETADDRINFO = 502,
    // /// `open` should call `openat`
    // #[cfg(feature = "fs")]
    // OPEN = 503,
    // /// This is not a syscall
    // #[cfg(feature = "multitask")]
    // PTHREAD_SELF = 504,
    // /// `pthread_create` should call `sys_clone`
    // #[cfg(feature = "multitask")]
    // PTHREAD_CREATE = 505,
    // /// Not a standard syscall
    // #[cfg(feature = "multitask")]
    // PTHREAD_EXIT = 506,
    // /// `pthread_join` should use `futex`
    // #[cfg(feature = "multitask")]
    // PTHREAD_JOIN = 507,
    // /// Not a standard syscall
    // #[cfg(feature = "multitask")]
    // PTHREAD_MUTEX_INIT = 508,
    // /// `pthread_mutex_lock` should call `futex`
    // #[cfg(feature = "multitask")]
    // PTHREAD_MUTEX_LOCK = 509,
    // /// `pthread_mutex_unlock` should call `futex`
    // #[cfg(feature = "multitask")]
    // PTHREAD_MUTEX_UNLOCK = 510,
}
