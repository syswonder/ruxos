/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Signal implementation, used by musl

use axerrno::LinuxError;

use crate::ctypes;
use core::{
    ffi::c_int,
    sync::atomic::{AtomicUsize, Ordering},
};

enum RTSigprocmaskHow {
    Block = 0,
    UnBlock = 1,
    SetMask = 2,
}

impl TryFrom<c_int> for RTSigprocmaskHow {
    type Error = ();
    fn try_from(value: c_int) -> Result<Self, Self::Error> {
        match value {
            x if x == Self::Block as c_int => Ok(Self::Block),
            x if x == Self::UnBlock as c_int => Ok(Self::UnBlock),
            x if x == Self::SetMask as c_int => Ok(Self::SetMask),
            _ => Err(()),
        }
    }
}

static mut MASK_TMP: AtomicUsize = AtomicUsize::new(0);

fn set_mask(old: *mut usize, new: usize) {
    unsafe {
        *old = new;
    }
}

fn get_mask(mask: *const usize) -> usize {
    unsafe { *mask }
}

/// Set mask for given thread
pub fn sys_rt_sigprocmask(
    how: c_int,
    _new_mask: *const usize,
    _old_mask: *mut usize,
    sigsetsize: usize,
) -> c_int {
    debug!(
        "sys_rt_sigprocmask <= flag: {}, sigsetsize: {}",
        how, sigsetsize
    );

    syscall_body!(sys_rt_sigprocmask, {
        if !_old_mask.is_null() {
            unsafe {
                let new = MASK_TMP.load(Ordering::Relaxed);
                set_mask(_old_mask, new);
            }
        }

        if !_new_mask.is_null() {
            unsafe {
                let set = get_mask(_new_mask);
                match how.try_into() {
                    Ok(RTSigprocmaskHow::Block) => MASK_TMP.fetch_or(set, Ordering::Relaxed),
                    Ok(RTSigprocmaskHow::UnBlock) => MASK_TMP.fetch_and(!set, Ordering::Relaxed),
                    Ok(RTSigprocmaskHow::SetMask) => MASK_TMP.swap(set, Ordering::Relaxed),
                    _ => return Err(LinuxError::EINVAL),
                };
            }
        }

        Ok(0)
    })
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
