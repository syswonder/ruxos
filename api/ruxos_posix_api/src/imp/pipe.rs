/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;
use ruxfs::fifo;

use axerrno::LinuxError;
use ruxfdtable::OpenFlags;

use ruxtask::fs::add_file_like;

/// Create a pipe
///
/// Return 0 if succeed
pub fn sys_pipe(fds: &mut [c_int]) -> c_int {
    sys_pipe2(fds, 0)
}

/// `pipe2` syscall, used by AARCH64
///
/// Return 0 on success
pub fn sys_pipe2(fds: &mut [c_int], flag: c_int) -> c_int {
    syscall_body!(sys_pipe2, {
        let flags = OpenFlags::from_bits(flag).ok_or(LinuxError::EINVAL)?;
        debug!(
            "sys_pipe2 <= fds: {:#x}, flag: {:?}",
            fds.as_ptr() as usize,
            flags
        );
        let (reader, writer) = fifo::new_pipe_pair(flags);
        fds[0] = add_file_like(reader, flags)?;
        fds[1] = add_file_like(writer, flags)?;
        debug!(
            "[sys_pipe] create pipe with read fd {} and write fd {}",
            fds[0], fds[1]
        );
        Ok(0)
    })
}
