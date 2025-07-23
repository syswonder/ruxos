/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Traphandle implementation
//!
//! Used to support musl syscall
#[cfg(feature = "musl")]
use crate::syscall_id::SyscallId;

/// Traphandler used by musl libc, overwrite handler in ruxruntime
struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl ruxhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(_irq_num: usize) {
        #[cfg(feature = "irq")]
        {
            let guard = kernel_guard::NoPreempt::new();
            ruxhal::irq::dispatch_irq(_irq_num);
            drop(guard); // rescheduling may occur when preemption is re-enabled.
        }
    }

    #[cfg(feature = "musl")]
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
        let id = SyscallId::try_from(syscall_id).unwrap_or(SyscallId::INVALID);
        if id == SyscallId::INVALID {
            info!("Invalid syscall id: {syscall_id}");
        }
        crate::syscall(id, args)
    }
}
