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
        crate::syscall(id, args)
    }
}
