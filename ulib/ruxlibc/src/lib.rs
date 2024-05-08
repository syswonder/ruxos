/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos] user program library for C apps.
//!
//! ## Cargo Features
//!
//! - CPU
//!     - `smp`: Enable SMP (symmetric multiprocessing) support.
//!     - `fp_simd`: Enable floating point and SIMD support.
//! - Interrupts:
//!     - `irq`: Enable interrupt handling support.
//! - Memory
//!     - `alloc`: Enable dynamic memory allocation.
//!     - `tls`: Enable thread-local storage.
//! - Task management
//!     - `multitask`: Enable multi-threading support.
//! - Upperlayer stacks
//!     - `fs`: Enable file system support.
//!     - `net`: Enable networking support.
//!     - `signal`: Enable signal support.
//! - Lib C functions
//!     - `fd`: Enable file descriptor table.
//!     - `pipe`: Enable pipe support.
//!     - `select`: Enable synchronous I/O multiplexing ([select]) support.
//!     - `epoll`: Enable event polling ([epoll]) support.
//!
//! [Ruxos]: https://github.com/syswonder/ruxos
//! [select]: https://man7.org/linux/man-pages/man2/select.2.html
//! [epoll]: https://man7.org/linux/man-pages/man7/epoll.7.html

#![cfg_attr(all(not(test), not(doc)), no_std)]
#![feature(doc_cfg)]
#![feature(doc_auto_cfg)]
#![feature(naked_functions)]
#![feature(thread_local)]
#![allow(clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;
#[cfg(feature = "alloc")]
mod env;
#[path = "."]
mod ctypes {
    #[rustfmt::skip]
    #[path = "libctypes_gen.rs"]
    #[allow(dead_code, non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::upper_case_acronyms)]
    mod libctypes;

    pub use libctypes::*;
    pub use ruxos_posix_api::ctypes::*;
}

#[macro_use]
mod utils;

#[cfg(feature = "fd")]
mod fd_ops;
#[cfg(feature = "fs")]
mod fs;
#[cfg(any(feature = "select", feature = "poll", feature = "epoll"))]
mod io_mpx;
#[cfg(feature = "alloc")]
mod malloc;
#[cfg(feature = "alloc")]
mod mmap;
#[cfg(feature = "net")]
mod net;
#[cfg(feature = "pipe")]
mod pipe;
#[cfg(feature = "multitask")]
mod pthread;
#[cfg(feature = "alloc")]
mod strftime;
#[cfg(feature = "fp_simd")]
mod strtod;

mod errno;
mod io;
mod mktime;
mod rand;
mod resource;
mod setjmp;
mod signal;
mod string;
mod sys;
mod time;
mod unistd;

#[cfg(not(test))]
pub use self::io::write;
pub use self::io::{read, writev};

pub use self::errno::strerror;
pub use self::mktime::mktime;
pub use self::rand::{getrandom, rand, random, srand};
pub use self::resource::{getrlimit, setrlimit};
pub use self::setjmp::{longjmp, setjmp};
pub use self::string::{strlen, strnlen};
pub use self::sys::sysconf;
pub use self::time::{clock_gettime, nanosleep};
pub use self::unistd::{abort, exit, getpid};

#[cfg(feature = "alloc")]
pub use self::env::{getenv, setenv, unsetenv};
#[cfg(feature = "fd")]
pub use self::fd_ops::{ax_fcntl, close, dup, dup2, dup3};
#[cfg(feature = "fs")]
pub use self::fs::{ax_open, fstat, getcwd, lseek, lstat, mkdir, rename, rmdir, stat, unlink};
#[cfg(feature = "fd")]
pub use self::io::rux_ioctl;
#[cfg(feature = "poll")]
pub use self::io_mpx::poll;
#[cfg(feature = "select")]
pub use self::io_mpx::select;
#[cfg(feature = "epoll")]
pub use self::io_mpx::{epoll_create, epoll_ctl, epoll_wait};
#[cfg(feature = "alloc")]
pub use self::malloc::{free, malloc};
#[cfg(feature = "alloc")]
pub use self::mmap::{mmap, munmap};
#[cfg(feature = "net")]
pub use self::net::{
    accept, ax_sendmsg, bind, connect, freeaddrinfo, getaddrinfo, getpeername, getsockname, listen,
    recv, recvfrom, send, sendto, shutdown, socket,
};
#[cfg(feature = "pipe")]
pub use self::pipe::pipe;
#[cfg(feature = "multitask")]
pub use self::pthread::{
    pthread_cond_broadcast, pthread_cond_init, pthread_cond_signal, pthread_cond_wait,
};
#[cfg(feature = "multitask")]
pub use self::pthread::{pthread_create, pthread_exit, pthread_join, pthread_self};
#[cfg(feature = "multitask")]
pub use self::pthread::{
    pthread_mutex_init, pthread_mutex_lock, pthread_mutex_trylock, pthread_mutex_unlock,
};
#[cfg(feature = "alloc")]
pub use self::strftime::strftime;
#[cfg(feature = "fp_simd")]
pub use self::strtod::{strtod, strtof};
#[cfg(feature = "signal")]
pub use self::time::{getitimer, setitimer};
#[cfg(feature = "signal")]
pub use self::unistd::{alarm, ualarm};
