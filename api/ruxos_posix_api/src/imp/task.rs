/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#[cfg(feature = "fd")]
use crate::imp::fd_ops::{fd_table_count, flush_file_like, RUX_FILE_LIMIT};
use core::ffi::c_int;

/// Relinquish the CPU, and switches to another task.
///
/// For single-threaded configuration (`multitask` feature is disabled), we just
/// relax the CPU and wait for incoming interrupts.
pub fn sys_sched_yield() -> c_int {
    #[cfg(feature = "multitask")]
    ruxtask::yield_now();
    #[cfg(not(feature = "multitask"))]
    if cfg!(feature = "irq") {
        ruxhal::arch::wait_for_irqs();
    } else {
        core::hint::spin_loop();
    }
    0
}

/// Get current thread ID.
pub fn sys_getpid() -> c_int {
    syscall_body!(sys_getpid,
        #[cfg(feature = "multitask")]
        {
            Ok(ruxtask::current().id().as_u64() as c_int)
        }
        #[cfg(not(feature = "multitask"))]
        {
            Ok(2) // `main` task ID
        }
    )
}

/// Exit current task
pub fn sys_exit(exit_code: c_int) -> ! {
    debug!("sys_exit <= {}", exit_code);
    #[cfg(feature = "fd")]
    {
        let mut now_fd_count = 0;
        let mut fd_index: usize = 0;
        while fd_index < RUX_FILE_LIMIT {
            let flush_res = flush_file_like(fd_index.try_into().unwrap());
            if flush_res == Ok(()) {
                now_fd_count += 1;
                if now_fd_count == fd_table_count() {
                    break;
                }
            }
            fd_index += 1;
        }
    }
    #[cfg(feature = "multitask")]
    ruxtask::exit(exit_code);
    #[cfg(not(feature = "multitask"))]
    ruxhal::misc::terminate();
}
