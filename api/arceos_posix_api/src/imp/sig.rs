//! Signal implementation

use core::ffi::c_int;

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
