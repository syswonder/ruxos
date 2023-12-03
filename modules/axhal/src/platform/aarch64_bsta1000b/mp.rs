/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::mem::{virt_to_phys, PhysAddr, VirtAddr};

/// Hart number of bsta1000b board
pub const MAX_HARTS: usize = 8;
/// CPU HWID from cpu device tree nodes with "reg" property
pub const CPU_HWID: [usize; MAX_HARTS] = [0x00, 0x100, 0x200, 0x300, 0x400, 0x500, 0x600, 0x700];

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(cpu_id: usize, stack_top: PhysAddr) {
    if cpu_id >= MAX_HARTS {
        error!("No support for bsta1000b core {}", cpu_id);
        return;
    }
    extern "C" {
        fn _start_secondary();
    }
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    crate::platform::aarch64_common::psci::cpu_on(
        CPU_HWID[cpu_id],
        entry.as_usize(),
        stack_top.as_usize(),
    );
}
