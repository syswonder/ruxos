/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Trap handling.
use crate_interface::{call_interface, def_interface};

/// Several reasons for page missing exceptions.
#[derive(Debug)]
pub enum PageFaultCause {
    /// pageFault caused by memory WRITE.
    WRITE,
    /// pageFault caused by memory READ.
    READ,
    /// pageFault caused by INSTRUCTION fetch.
    INSTRUCTION,
}

/// Trap handler interface.
///
/// This trait is defined with the [`#[def_interface]`][1] attribute. Users
/// should implement it with [`#[impl_interface]`][2] in any other crate.
///
/// [1]: crate_interface::def_interface
/// [2]: crate_interface::impl_interface
#[def_interface]
pub trait TrapHandler {
    /// Handles interrupt requests for the given IRQ number.
    fn handle_irq(_irq_num: usize) {
        panic!("No handle_irq implement");
    }
    /// Handles system call from user app.
    #[cfg(feature = "musl")]
    fn handle_syscall(_syscall_id: usize, _args: [usize; 6]) -> isize {
        panic!("No handle_syscall implement");
    }
    /// Handles page fault for mmap.
    #[cfg(feature = "paging")]
    fn handle_page_fault(_vaddr: usize, _caus: PageFaultCause) -> bool {
        panic!("No handle_page_fault implement");
    }
}

/// Call the external IRQ handler.
#[allow(dead_code)]
pub(crate) fn handle_irq_extern(irq_num: usize) {
    call_interface!(TrapHandler::handle_irq, irq_num);
}

/// Call the external syscall handler.
#[allow(dead_code)]
#[cfg(feature = "musl")]
pub(crate) fn handle_syscall(syscall_id: usize, args: [usize; 6]) -> isize {
    call_interface!(TrapHandler::handle_syscall, syscall_id, args)
}

/// Call the external IRQ handler.
#[allow(dead_code)]
#[cfg(feature = "paging")]
pub(crate) fn handle_page_fault(vaddr: usize, cause: PageFaultCause) -> bool {
    call_interface!(TrapHandler::handle_page_fault, vaddr, cause)
}
