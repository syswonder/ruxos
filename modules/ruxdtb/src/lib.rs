/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Parsing specific flat device tree field for AArch64.
#![no_std]

#[macro_use]
#[cfg(target_arch = "aarch64")]
extern crate log;

/// get memory base and size from dtb
///
/// # Return
/// Return `(mem_base, mem_size)`
#[cfg(target_arch = "aarch64")]
pub fn get_memory_info() -> (usize, usize) {
    let mem_node = dtb::get_node_by_prop_value("device_type", "memory").unwrap();

    let regs = mem_node.find_prop("reg").unwrap();
    (
        dtb::get_propbuf_u64(&regs, 0) as usize,
        dtb::get_propbuf_u64(&regs, 1) as usize,
    )
}

/// get memory base and size from dtb
///
/// # Return
/// Return `(mem_base, mem_size)`
#[cfg(not(target_arch = "aarch64"))]
pub fn get_memory_info() -> (usize, usize) {
    (0, 0)
}

/// Get pl011 base address from dtb
#[cfg(target_arch = "aarch64")]
pub fn get_pl011_base() -> usize {
    let pl011_node = dtb::compatible_node("arm,pl011").unwrap();
    let regs = pl011_node.find_prop("reg").unwrap();
    dtb::get_propbuf_u64(&regs, 0) as usize
}

/// Get pl011 base address from dtb
#[cfg(not(target_arch = "aarch64"))]
pub fn get_pl011_base() -> usize {
    0
}

/// Get pl031 base address from dtb
#[cfg(target_arch = "aarch64")]
pub fn get_pl031_base() -> usize {
    let pl031_node = dtb::compatible_node("arm,pl031").unwrap();
    let regs = pl031_node.find_prop("reg").unwrap();
    dtb::get_propbuf_u64(&regs, 0) as usize
}

/// Get pl031 base address from dtb
#[cfg(not(target_arch = "aarch64"))]
pub fn get_pl031_base() -> usize {
    0
}

/// Get GICV2 base address from dtb
#[cfg(all(target_arch = "aarch64", feature = "irq"))]
pub fn get_gicv3_base() -> usize {
    let gicv3_node = dtb::compatible_node("arm,gic-v3").unwrap();
    let regs = gicv3_node.find_prop("reg").unwrap();
    // index 0 for GICD
    dtb::get_propbuf_u64(&regs, 0) as usize
}

/// Get GICV2 base address from dtb
#[cfg(not(target_arch = "aarch64"))]
pub fn get_gicv3_base() -> usize {
    0
}

/// Get GICV2 base address from dtb
#[cfg(all(target_arch = "aarch64", feature = "irq"))]
pub fn get_gicv2_base() -> usize {
    let gicv2_node = dtb::compatible_node("arm,cortex-a15-gic").unwrap();
    let regs = gicv2_node.find_prop("reg").unwrap();
    // index 0 for GICD
    dtb::get_propbuf_u64(&regs, 0) as usize
}

/// Get GICV2 base address from dtb
#[cfg(not(target_arch = "aarch64"))]
pub fn get_gicv2_base() -> usize {
    0
}

/// Get psci version from dtb
#[cfg(target_arch = "aarch64")]
pub fn get_psci() {
    let psci_node = match dtb::compatible_node("arm,psci-1.0") {
        Some(node) => node,
        None => dtb::compatible_node("arm,psci-0.2").unwrap(),
    };
    let prop = psci_node.find_prop("method").unwrap().str();
    info!("prop: {:?}", prop);
}

/// Get psci version from dtb
#[cfg(not(target_arch = "aarch64"))]
pub fn get_psci() {}
