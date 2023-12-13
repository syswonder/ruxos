/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;

#[cfg(feature = "signal")]
use crate::ctypes::k_sigaction;
use crate::ctypes::{sigaction, EINVAL, SIGKILL, SIGSTOP};
#[cfg(feature = "signal")]
use ruxos_posix_api::sys_sigaction;

#[cfg(feature = "signal")]
unsafe extern "C" fn ignore_handler(_: c_int) {}

#[no_mangle]
pub unsafe extern "C" fn sigaction_inner(
    signum: c_int,
    _act: *const sigaction,
    oldact: *mut sigaction,
) -> c_int {
    if signum >= 32 || signum == SIGKILL as _ || signum == SIGSTOP as _ {
        return -(EINVAL as c_int);
    }
    #[cfg(feature = "signal")]
    {
        let mut sh = (*_act).__sa_handler.sa_handler;
        if let Some(h) = sh {
            if h as usize == crate::ctypes::SIGIGN as usize {
                sh = Some(ignore_handler as unsafe extern "C" fn(c_int));
            }
        }
        let k_act = k_sigaction {
            handler: sh,
            flags: (*_act).sa_flags as _,
            restorer: (*_act).sa_restorer,
            mask: Default::default(),
        };
        let mut k_oldact = k_sigaction::default();
        sys_sigaction(
            signum as u8,
            Some(&k_act),
            if oldact.is_null() {
                None
            } else {
                Some(&mut k_oldact)
            },
        );
        if !oldact.is_null() {
            (*oldact).__sa_handler.sa_handler = k_oldact.handler;
            (*oldact).sa_flags = k_oldact.flags as _;
            (*oldact).sa_restorer = k_oldact.restorer;
            // Not support mask
            // (*oldact).sa_mask = k_oldact.mask;
        }
    }
    #[cfg(not(feature = "signal"))]
    {
        if !oldact.is_null() {
            // set to 0
            (*oldact).__sa_handler.sa_handler = None;
            (*oldact).sa_flags = 0;
            (*oldact).sa_restorer = None;
            (*oldact).sa_mask = Default::default();
        }
    }
    0
}
