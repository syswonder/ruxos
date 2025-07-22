/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::ffi::c_int;

#[derive(Debug, Clone, Copy)]
struct UserCapHeader {
    /// Linux Cap Version:
    /// Version1 = 0x19980330,
    /// Version2 = 0x20071026,
    /// Version3 = 0x20080522,
    version: u32,
    pid: i32,
}

/// The effective, permitted, and inheritable fields are bit masks of the capabilities.  
/// Note that the CAP_* values are bit indexes and need to be bit-shifted before ORing into the bit fields.
#[derive(Debug, Clone, Copy)]
struct UserCapData {
    effective: u32,
    permitted: u32,
    inheritable: u32,
}

/// get thread capabilities. specific to Linux.
pub fn sys_cap_get(cap_user_header: usize, cap_user_data: usize) -> c_int {
    let hdrp = cap_user_header as *const UserCapHeader;
    let datap = cap_user_data as *mut UserCapData;
    unsafe {
        debug!(
            "sys_cap_get <= pid {:?}, version {:x?} ",
            (*hdrp).pid,
            (*hdrp).version
        );
    }
    syscall_body!(sys_cap_get, {
        unsafe {
            // allow all
            (*datap).effective = u32::MAX;
            (*datap).inheritable = u32::MAX;
            (*datap).permitted = u32::MAX;
        };
        Ok(0)
    })
}
