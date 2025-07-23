/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::{c_int, c_ulong};

use axerrno::LinuxError;

const ARCH_SET_FS: i32 = 0x1002;

/// set thread state
pub fn sys_arch_prctl(code: c_int, addr: c_ulong) -> c_int {
    debug!("sys_arch_prctl <= code: {code}, addr: {addr:#x}");
    syscall_body!(sys_arch_prctl, {
        match code {
            ARCH_SET_FS => {
                unsafe {
                    ruxhal::arch::write_thread_pointer(addr as _);
                }
                Ok(0)
            }
            _ => Err(LinuxError::EINVAL),
        }
    })
}

/// TODO: fake implementation for prctl
pub fn sys_prctl(op: c_int, arg0: c_ulong, arg1: c_ulong, arg2: c_ulong, arg3: c_ulong) -> c_int {
    debug!("sys_prctl <= op: {op}, arg0: {arg0}, arg1: {arg1}, arg2: {arg2}, arg3: {arg3}");
    syscall_body!(sys_prctl, Ok(0))
}
