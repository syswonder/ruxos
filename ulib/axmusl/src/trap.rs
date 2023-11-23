//! Traphandle implementation
//!
//! Used to support musl syscall

use crate::syscall::syscall_id::SyscallId;

/// Traphandler used by musl libc, overwrite handler in axruntime
struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl axhal::trap::TrapHandler for TrapHandlerImpl {
    fn handle_irq(_irq_num: usize) {
        #[cfg(feature = "irq")]
        {
            let guard = kernel_guard::NoPreempt::new();
            axhal::irq::dispatch_irq(_irq_num);
            drop(guard); // rescheduling may occur when preemption is re-enabled.
        }
    }

    #[cfg(feature = "musl")]
    fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
        let id = SyscallId::try_from(syscall_id).unwrap_or(SyscallId::INVALID);
        crate::syscall::syscall(id, args)
    }
}
