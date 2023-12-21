/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::mem::{virt_to_phys, PhysAddr, VirtAddr};

/// Starts the given secondary CPU with its boot stack.
pub fn start_secondary_cpu(hartid: usize, stack_top: PhysAddr) {
    extern "C" {
        fn _start_secondary();
    }
    if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
        warn!("HSM SBI extension is not supported for current SEE.");
        return;
    }
    let entry = virt_to_phys(VirtAddr::from(_start_secondary as usize));
    sbi_rt::hart_start(hartid, entry.as_usize(), stack_top.as_usize());
}
