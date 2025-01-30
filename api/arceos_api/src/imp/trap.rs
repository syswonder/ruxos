/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use ruxhal::trap::{PageFaultCause, TrapHandler};

struct TrapHandlerImpl;

#[crate_interface::impl_interface]
impl TrapHandler for TrapHandlerImpl {
    fn handle_page_fault(vaddr: usize, cause: PageFaultCause) -> bool {
        // TODO: handle page fault
        panic!("Page fault at {:#x} with cause {:?}.", vaddr, cause);
    }
}
