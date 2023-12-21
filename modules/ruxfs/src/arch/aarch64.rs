/* Copyright (c) [2023] [Syswonder Community]
 *   [Rukos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::format;
use alloc::string::String;

fn read_cpuid() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, midr_el1", out(reg) val);
    }
    val
}

pub fn get_cpuinfo() -> String {
    let cpuid = read_cpuid();
    let mut cpuinfo = String::new();

    cpuinfo.push_str(
        format!(
            "Processor\t: {} rev {} ({})\n",
            "AArch64 Processor",
            cpuid & 15,
            "aarch64"
        )
        .as_ref(),
    );
    cpuinfo.push_str("Features\t: ");
    #[cfg(feature = "fp_simd")]
    cpuinfo.push_str("fp asimd");

    cpuinfo.push_str(format!("\nCPU implementer\t: {:#02x}\n", cpuid >> 24).as_ref());
    cpuinfo.push_str("CPU architecture: AArch64\n");
    cpuinfo.push_str(format!("CPU variant\t: {:#x}\n", (cpuid >> 20) & 15).as_ref());
    cpuinfo.push_str(format!("CPU part\t: {:#03x}\n", (cpuid >> 4) & 0xfff).as_ref());
    cpuinfo.push_str(format!("CPU revision\t: {}\n", cpuid & 15).as_ref());

    cpuinfo
}
