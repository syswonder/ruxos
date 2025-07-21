/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use ruxfdtable::OpenFlags;
use ruxtask::fs::get_file_like;

pub const FIONBIO: usize = 0x5421;
pub const FIOCLEX: usize = 0x5451;

/// ioctl implementation
pub fn sys_ioctl(fd: c_int, request: usize, data: usize) -> c_int {
    debug!("sys_ioctl <= fd: {fd}, request: {request}");
    syscall_body!(sys_ioctl, {
        match request {
            FIONBIO => {
                let f = get_file_like(fd)?;
                let flags = if unsafe { *(data as *const i32) } > 0 {
                    f.flags() | OpenFlags::O_NONBLOCK
                } else {
                    f.flags() & !OpenFlags::O_NONBLOCK
                };
                f.set_flags(flags)?;
                Ok(0)
            }
            FIOCLEX => Ok(0),
            _ => {
                get_file_like(fd)?.ioctl(request, data)?;
                Ok(0)
            }
        }
    })
}
