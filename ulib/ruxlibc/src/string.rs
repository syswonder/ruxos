/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes;
use core::ffi::c_char;
/// calculate the length of a string, excluding the terminating null byte
#[no_mangle]
pub unsafe extern "C" fn strlen(s: *const c_char) -> ctypes::size_t {
    strnlen(s, ctypes::size_t::MAX)
}

/// calculate the length of a string like strlen, but at most maxlen.
#[no_mangle]
pub unsafe extern "C" fn strnlen(s: *const c_char, size: ctypes::size_t) -> ctypes::size_t {
    let mut i = 0;
    while i < size {
        if *s.add(i) == 0 {
            break;
        }
        i += 1;
    }
    i
}
