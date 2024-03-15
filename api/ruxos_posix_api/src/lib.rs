/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! POSIX-compatible APIs for [Ruxos] modules
//!
//! [Ruxos]: https://github.com/syswonder/ruxos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(ip_in_core)]
#![feature(result_option_inspect)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate axlog;
extern crate ruxruntime;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use ruxruntime::{environ, environ_iter, RUX_ENVIRON};

#[macro_use]
mod utils;

mod imp;

/// Platform-specific constants and parameters.
pub mod config {
    pub use memory_addr::PAGE_SIZE_4K;
    pub use ruxconfig::*;
}

/// POSIX C types.
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms, missing_docs)]
pub mod ctypes;

pub use imp::getrandom::{sys_getrandom, sys_rand, sys_random, sys_srand};
pub use imp::io::{sys_read, sys_readv, sys_write, sys_writev};
pub use imp::prctl::{sys_arch_prctl, sys_prctl};
pub use imp::resources::{sys_getrlimit, sys_prlimit64, sys_setrlimit};
pub use imp::rt_sig::{sys_rt_sigaction, sys_rt_sigprocmask};
pub use imp::stat::{sys_getegid, sys_geteuid, sys_umask};
pub use imp::sys::{sys_sysinfo, sys_uname};
pub use imp::sys_invalid;
pub use imp::task::{sys_exit, sys_getpid, sys_sched_yield};
pub use imp::time::{sys_clock_gettime, sys_clock_settime, sys_gettimeofday, sys_nanosleep};

#[cfg(all(feature = "fd", feature = "musl"))]
pub use imp::fd_ops::sys_dup3;
#[cfg(feature = "fd")]
pub use imp::fd_ops::{sys_close, sys_dup, sys_dup2, sys_fcntl};
#[cfg(feature = "fs")]
pub use imp::fs::{
    sys_fchownat, sys_fdatasync, sys_fstat, sys_fsync, sys_getcwd, sys_getdents64, sys_lseek,
    sys_lstat, sys_mkdir, sys_mkdirat, sys_newfstatat, sys_open, sys_openat, sys_pread, sys_preadv,
    sys_readlinkat, sys_rename, sys_renameat, sys_rmdir, sys_stat, sys_unlink, sys_unlinkat,
};
#[cfg(feature = "epoll")]
pub use imp::io_mpx::{sys_epoll_create, sys_epoll_ctl, sys_epoll_pwait, sys_epoll_wait};
#[cfg(feature = "poll")]
pub use imp::io_mpx::{sys_poll, sys_ppoll};
#[cfg(feature = "select")]
pub use imp::io_mpx::{sys_pselect6, sys_select};
#[cfg(feature = "fd")]
pub use imp::ioctl::sys_ioctl;
#[cfg(feature = "alloc")]
pub use imp::mmap::{sys_madvise, sys_mmap, sys_mprotect, sys_mremap, sys_munmap};
#[cfg(feature = "net")]
pub use imp::net::{
    sys_accept, sys_bind, sys_connect, sys_freeaddrinfo, sys_getaddrinfo, sys_getpeername,
    sys_getsockname, sys_listen, sys_recv, sys_recvfrom, sys_send, sys_sendmsg, sys_sendto,
    sys_setsockopt, sys_shutdown, sys_socket,
};
#[cfg(feature = "pipe")]
pub use imp::pipe::{sys_pipe, sys_pipe2};
#[cfg(feature = "multitask")]
pub use imp::pthread::condvar::{
    sys_pthread_cond_broadcast, sys_pthread_cond_destroy, sys_pthread_cond_init,
    sys_pthread_cond_signal, sys_pthread_cond_timedwait, sys_pthread_cond_wait,
};
#[cfg(feature = "multitask")]
pub use imp::pthread::mutex::{
    sys_pthread_mutex_destroy, sys_pthread_mutex_init, sys_pthread_mutex_lock,
    sys_pthread_mutex_trylock, sys_pthread_mutex_unlock,
};
#[cfg(feature = "multitask")]
pub use imp::pthread::{
    sys_pthread_getspecific, sys_pthread_key_create, sys_pthread_key_delete,
    sys_pthread_setspecific,
};
#[cfg(feature = "signal")]
pub use imp::signal::{sys_getitimer, sys_setitimer, sys_sigaction, sys_sigaltstack};

#[cfg(feature = "multitask")]
pub use imp::pthread::futex::sys_futex;
#[cfg(all(
    feature = "multitask",
    feature = "musl",
))]
pub use imp::pthread::sys_clone;
#[cfg(all(feature = "multitask", feature = "musl"))]
pub use imp::pthread::sys_set_tid_address;
#[cfg(feature = "multitask")]
pub use imp::pthread::{sys_pthread_create, sys_pthread_exit, sys_pthread_join, sys_pthread_self};
