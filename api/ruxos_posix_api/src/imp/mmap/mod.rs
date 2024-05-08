/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

cfg_if::cfg_if! {
    if #[cfg(all(feature = "paging", target_arch = "aarch64"))] {
        #[macro_use]
        mod utils;
        mod api;
        mod trap;
        pub use self::api::{sys_madvise, sys_mmap, sys_mprotect, sys_mremap, sys_msync, sys_munmap};
    }else {
        mod legacy;
        pub use self::legacy::{sys_madvise, sys_mmap, sys_mprotect, sys_mremap, sys_msync, sys_munmap};
    }
}
