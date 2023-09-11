//! Signal implementation, used by musl

use core::ffi::c_int;

use crate::ctypes;

/// Set mask for given thread
pub fn sys_rt_sigprocmask(
    flag: c_int,
    _new_mask: *const usize,
    _old_mask: *mut usize,
    sigsetsize: usize,
) -> c_int {
    debug!(
        "sys_rt_sigprocmask <= flag: {}, sigsetsize: {}",
        flag, sigsetsize
    );
    syscall_body!(sys_rt_sigprocmask, Ok(0))
}

/// sigaction syscall for A64 musl
pub fn sys_rt_sigaction(
    sig: c_int,
    _sa: *const ctypes::sigaction,
    _old: *mut ctypes::sigaction,
    _sigsetsize: ctypes::size_t,
) -> c_int {
    debug!("sys_rt_sigaction <= sig: {}", sig);
    syscall_body!(sys_rt_sigaction, Ok(0))
}
