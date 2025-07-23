/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::format;
use alloc::string::String;

#[cfg(target_arch = "x86_64")]
mod x86_64;

#[cfg(target_arch = "aarch64")]
mod aarch64;

pub fn get_cpuinfo() -> String {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "aarch64")] {
            aarch64::get_cpuinfo()
        } else if #[cfg(target_arch = "x86_64")] {
            x86_64::get_cpuinfo()
        } else {
            String::new()
        }
    }
}

pub fn get_meminfo() -> String {
    #[cfg(feature = "alloc")]
    {
        use core::ffi::c_ulong;
        let allocator = axalloc::global_allocator();
        let freeram = (allocator.available_bytes()
            + allocator.available_pages() * memory_addr::PAGE_SIZE_4K)
            as c_ulong;
        let totalram = freeram + allocator.used_bytes() as c_ulong;

        let mut meminfo = String::new();
        meminfo.push_str(format!("MemTotal:       {totalram:8}\n").as_ref());
        meminfo.push_str(format!("MemFree:        {freeram:8}\n").as_ref());

        meminfo
    }
    #[cfg(not(feature = "alloc"))]
    String::new()
}
