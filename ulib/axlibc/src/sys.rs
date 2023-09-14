/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use arceos_posix_api::{config, ctypes, sys_getrlimit, PAGE_SIZE_4K};
use core::ffi::{c_int, c_long};

/// Return system configuration infomation
///
/// Notice: currently only support what unikraft covers
#[no_mangle]
pub unsafe extern "C" fn sysconf(name: c_int) -> c_long {
    match name as u32 {
        // Maximum process number
        ctypes::_SC_CHILD_MAX => {
            let mut rl: ctypes::rlimit = core::mem::zeroed();
            sys_getrlimit(ctypes::RLIMIT_NPROC.try_into().unwrap(), &mut rl);
            rl.rlim_max as c_long
        }
        // Page size
        ctypes::_SC_PAGE_SIZE => PAGE_SIZE_4K as c_long,
        // Total physical pages
        ctypes::_SC_PHYS_PAGES => (config::PHYS_MEMORY_SIZE / PAGE_SIZE_4K) as c_long,
        // Number of processors in use
        ctypes::_SC_NPROCESSORS_ONLN => config::SMP as c_long,
        // Avaliable physical pages
        ctypes::_SC_AVPHYS_PAGES => {
            let mut info: arceos_posix_api::ctypes::sysinfo = core::mem::zeroed();
            arceos_posix_api::sys_sysinfo(&mut info);
            (info.freeram / PAGE_SIZE_4K as u64) as c_long
        }
        // Maximum number of files per process
        #[cfg(feature = "fd")]
        ctypes::_SC_OPEN_MAX => {
            let mut rl: ctypes::rlimit = core::mem::zeroed();
            sys_getrlimit(ctypes::RLIMIT_NOFILE.try_into().unwrap(), &mut rl);
            rl.rlim_max as c_long
        }
        _ => 0,
    }
}
