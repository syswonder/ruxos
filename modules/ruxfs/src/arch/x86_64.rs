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
use core::arch::x86_64::__cpuid;

#[derive(Default)]
struct CpuinfoX86 {
    x86_family: u8,
    x86_model: u8,
    x86_mask: u8,
    cpuid_level: i32, // Maximum supported CPUID level, -1=no CPUID
    x86_model_id: [u32; 16],
    extended_cpuid_level: u32, // Max extended CPUID function supported
}

impl CpuinfoX86 {
    fn get_model_name(&self) -> [u8; 64] {
        let mut model_name = [0; 64];
        for i in 0..16 {
            let bytes = self.x86_model_id[i].to_le_bytes();
            model_name[i * 4..i * 4 + 4].copy_from_slice(bytes.as_ref());
        }

        model_name
    }
}

/// Identifies and stores CPU's family, model, mask,
/// and the highest supported standard and extended CPUID levels.
fn cpu_dect(c: &mut CpuinfoX86) {
    // Get vendor name
    let eax = read_cpuid_eax(0x00000000);
    c.cpuid_level = eax as i32;

    // Set default x86 family
    c.x86_family = 4;
    if c.cpuid_level >= 0x00000001 {
        let tfms = read_cpuid_eax(0x00000001);
        c.x86_family = (tfms >> 8) as u8 & 0xf;
        c.x86_model = (tfms >> 4) as u8 & 0xf;
        c.x86_mask = tfms as u8 & 0xf;

        if c.x86_family == 0xf {
            c.x86_family += (tfms >> 20) as u8;
        }
        if c.x86_family >= 0x6 {
            c.x86_model += ((tfms >> 16) as u8 & 0xf) << 4;
        }
    }

    c.extended_cpuid_level = read_cpuid_eax(0x80000000);
}

fn gen_model_name(c: &mut CpuinfoX86) {
    if c.extended_cpuid_level < 0x80000004 {
        return;
    }

    let v = &mut c.x86_model_id;
    (v[0], v[1], v[2], v[3]) = read_cpuid(0x80000002);
    (v[4], v[5], v[6], v[7]) = read_cpuid(0x80000003);
    (v[8], v[9], v[10], v[11]) = read_cpuid(0x80000004);

    v[12] &= 0xffff_ff00;
}

fn read_cpuid(eax: u32) -> (u32, u32, u32, u32) {
    let cpuid_result = unsafe { __cpuid(eax) };

    (
        cpuid_result.eax,
        cpuid_result.ebx,
        cpuid_result.ecx,
        cpuid_result.edx,
    )
}

fn read_cpuid_eax(eax: u32) -> u32 {
    let cpuid_result = unsafe { __cpuid(eax) };

    cpuid_result.eax
}

pub fn get_cpuinfo() -> String {
    let mut c = CpuinfoX86::default();
    cpu_dect(&mut c);
    gen_model_name(&mut c);

    let mut cpuinfo = String::new();
    cpuinfo.push_str(format!("processor\t: {}\n", 0).as_ref());
    cpuinfo.push_str(format!("cpu family\t: {}\n", c.x86_family).as_ref());
    cpuinfo.push_str(format!("model\t\t: {}\n", c.x86_model).as_ref());

    let model_bytes = c.get_model_name();
    if model_bytes[0] == 0 {
        cpuinfo.push_str("model name\t: unknown\n".as_ref());
    } else {
        let model_name = String::from_utf8_lossy(&model_bytes);
        cpuinfo.push_str(format!("model name\t: {}\n", model_name).as_ref());
    }

    if c.x86_mask != 0 || c.cpuid_level >= 0 {
        cpuinfo.push_str(format!("stepping\t: {}\n", c.x86_mask).as_ref());
    } else {
        cpuinfo.push_str("stepping\t: unknown\n".as_ref());
    }

    cpuinfo
}
