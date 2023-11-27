/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::e;
use core::ffi::{c_char, c_int};
use ruxos_posix_api::ctypes;
use ruxos_posix_api::{sys_freeaddrinfo, sys_getaddrinfo};

/// Query addresses for a domain name.
///
/// Return address number if success.
#[no_mangle]
pub unsafe extern "C" fn getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    hints: *const ctypes::addrinfo,
    res: *mut *mut ctypes::addrinfo,
) -> c_int {
    let ret = e(sys_getaddrinfo(nodename, servname, hints, res));
    match ret {
        r if r < 0 => ctypes::EAI_FAIL,
        0 => ctypes::EAI_NONAME,
        _ => 0,
    }
}

/// Free queried `addrinfo` struct
#[no_mangle]
pub unsafe extern "C" fn freeaddrinfo(res: *mut ctypes::addrinfo) {
    sys_freeaddrinfo(res);
}
