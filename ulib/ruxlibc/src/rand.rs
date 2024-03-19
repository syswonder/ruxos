/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::ctypes::size_t;
use core::ffi::{c_int, c_long, c_uint, c_void};

use ruxos_posix_api::{sys_getrandom, sys_rand, sys_random, sys_srand};

use crate::utils::e;

/// srand
#[no_mangle]
pub unsafe extern "C" fn srand(seed: c_uint) {
    sys_srand(seed);
}

/// rand
#[no_mangle]
pub unsafe extern "C" fn rand() -> c_int {
    e(sys_rand() as c_int)
}

/// random
#[no_mangle]
pub unsafe extern "C" fn random() -> c_long {
    e(sys_random().try_into().unwrap()) as _
}

/// Get random
#[no_mangle]
pub unsafe extern "C" fn getrandom(buf: *mut c_void, buflen: size_t, flags: c_int) -> size_t {
    e(sys_getrandom(buf, buflen, flags).try_into().unwrap()) as _
}
