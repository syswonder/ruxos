/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! POSIX-compatible APIs for [ArceOS] modules
//!
//! [ArceOS]: https://github.com/rcore-os/arceos

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(ip_in_core)]
#![feature(result_option_inspect)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![allow(clippy::missing_safety_doc)]
#![feature(c_size_t)]

#[macro_use]
extern crate axlog;
extern crate axruntime;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use axruntime::{environ, environ_iter, RX_ENVIRON};

#[macro_use]
mod utils;

mod imp;

/// Platform-specific constants and parameters.
pub mod config {
    pub use axconfig::*;
    pub use memory_addr::PAGE_SIZE_4K;
}

/// POSIX C types.
#[rustfmt::skip]
#[path = "./ctypes_gen.rs"]
#[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms, missing_docs)]
pub mod ctypes;

pub use imp::io::{sys_read, sys_write, sys_writev, sys_ioctl};
pub use imp::resources::{sys_getrlimit, sys_setrlimit};
#[cfg(feature = "signal")]
pub use imp::signal::sys_sigaction;
pub use imp::sys::sys_sysinfo;
pub use imp::task::{sys_exit, sys_getpid, sys_sched_yield};
pub use imp::time::{sys_clock_gettime, sys_clock_settime, sys_nanosleep};
#[cfg(feature = "signal")]
pub use imp::time::{sys_getitimer, sys_setitimer};

#[cfg(feature = "fd")]
pub use imp::fd_ops::{sys_close, sys_dup, sys_dup2, sys_fcntl};
#[cfg(feature = "fs")]
pub use imp::fs::{
    sys_fstat, sys_getcwd, sys_lseek, sys_lstat, sys_mkdir, sys_open, sys_rename, sys_rmdir,
    sys_stat, sys_unlink
};
#[cfg(feature = "poll")]
pub use imp::io_mpx::sys_poll;
#[cfg(feature = "select")]
pub use imp::io_mpx::sys_select;
#[cfg(feature = "epoll")]
pub use imp::io_mpx::{sys_epoll_create, sys_epoll_ctl, sys_epoll_wait};
#[cfg(feature = "net")]
pub use imp::net::{
    sys_accept, sys_bind, sys_connect, sys_freeaddrinfo, sys_getaddrinfo, sys_getpeername,
    sys_getsockname, sys_listen, sys_recv, sys_recvfrom, sys_send, sys_sendmsg, sys_sendto,
    sys_shutdown, sys_socket,
};
#[cfg(feature = "pipe")]
pub use imp::pipe::sys_pipe;
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
pub use imp::pthread::tsd::{
    sys_pthread_getspecific, sys_pthread_key_create, sys_pthread_key_delete,
    sys_pthread_setspecific,
};
#[cfg(feature = "multitask")]
pub use imp::pthread::{sys_pthread_create, sys_pthread_exit, sys_pthread_join, sys_pthread_self};
