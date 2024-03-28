/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

/// If adding `random-hw` to features, rand()/random() will try to use CPU instruction to generate the random.
/// If CPU doesn't support instructions for random Generation, rand()/random() will return persudo random using LCG algorithm instead;
/// Without feature `random-hw`, rand()/random() will simply return 64-bit persudo random generated from LCG algorithm by default.
/// For x86_64, intel's CPU support `rdrand` instruction since IvyBridge, AMD's CPU support `rdrand` since Ryzen.
/// For aarch64, resigter `rndr` is supported for part of CPU core since ARMv8.5. For more information, you can read: https://developer.arm.com/documentation/ddi0601/2023-06/AArch64-Registers/RNDR--Random-Number?lang=en
/// We can determine whether the CPU supports this instruction by CPUID(x86_64) or ID_AA64ISAR0_EL1(aarch), which is implement in function `has_rdrand()`.
/// As of now, riscv64 does not support generating random numbers through instructions.           
use core::ffi::{c_int, c_long, c_uint, c_void};

use core::sync::atomic::{AtomicU64, Ordering::SeqCst};

use crate::ctypes::{size_t, ssize_t};

use axerrno::LinuxError;

#[cfg(all(target_arch = "x86_64", feature = "random-hw"))]
use core::arch::x86_64::__cpuid;

static SEED: AtomicU64 = AtomicU64::new(0xae_f3);

/// Returns a 32-bit unsigned pseudo random interger using LCG.
fn rand_lcg32() -> u32 {
    let new_seed = SEED
        .load(SeqCst)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    SEED.store(new_seed, SeqCst);
    (new_seed >> 33) as u32
}

/// Returns a 64-bit unsigned pseudo random interger using LCG.
fn random_lcg64() -> u64 {
    let new_seed = SEED
        .load(SeqCst)
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1);
    SEED.store(new_seed, SeqCst);
    new_seed >> 1
}

/// Sets the seed for the random number generator implemented by LCG.
fn srand_lcg(seed: u64) {
    SEED.store(seed - 1, SeqCst);
}

/// Checking if the CPU core is compatible with hardware random number instructions.
#[cfg(feature = "random-hw")]
fn has_rdrand() -> bool {
    #[cfg(target_arch = "x86_64")]
    {
        unsafe { __cpuid(1).ecx & (1 << 30) != 0 }
    }
    #[cfg(target_arch = "aarch64")]
    {
        let mut id_aa64_isar0_el1: u64;
        unsafe {
            core::arch::asm!(
                "mrs {},ID_AA64ISAR0_EL1",
                out(reg) id_aa64_isar0_el1
            )
        }
        id_aa64_isar0_el1 & (0b1111 << 60) == 0b0001 << 60
    }
    #[cfg(target_arch = "riscv64")]
    {
        false
    }
}

/// Return 64-bit unsigned random interger using cpu instruction
#[cfg(feature = "random-hw")]
fn random_hw() -> u64 {
    let mut _random: u64;

    #[cfg(target_arch = "x86_64")]
    {
        unsafe {
            core::arch::asm! {
                "rdrand {0:r}",
                out(reg) _random
            }
        }
        _random
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            core::arch::asm! {
                "mrs {}, s3_3_c2_c4_0", // s3_3_c2_c4_0 is register `rndr`
                out(reg) _random
            }
        }
        _random
    }

    #[cfg(target_arch = "riscv64")]
    {
        panic!("riscv64 has no rdrand instructions")
    }
}

/// Sets the seed for the 32-bit random number generator based on LCG.
#[no_mangle]
pub unsafe extern "C" fn sys_srand(_seed: c_uint) {
    srand_lcg(_seed as u64);
}

/// Returns a 32-bit unsigned random integer
#[no_mangle]
pub unsafe extern "C" fn sys_rand() -> c_int {
    #[cfg(feature = "random-hw")]
    {
        match has_rdrand() {
            true => (random_hw() >> 33) as c_int,
            false => rand_lcg32() as c_int,
        }
    }
    #[cfg(not(feature = "random-hw"))]
    {
        rand_lcg32() as c_int
    }
}

/// Returns a 64-bit unsigned random integer
#[no_mangle]
pub unsafe extern "C" fn sys_random() -> c_long {
    #[cfg(feature = "random-hw")]
    {
        match has_rdrand() {
            true => (random_hw() >> 1) as c_long,
            false => random_lcg64() as c_long,
        }
    }
    #[cfg(not(feature = "random-hw"))]
    {
        random_lcg64() as c_long
    }
}

/// Fills the buffer pointed to by buf with up to buflen random bytes.
pub unsafe extern "C" fn sys_getrandom(buf: *mut c_void, buflen: size_t, flags: c_int) -> ssize_t {
    debug!(
        "sys_getrandom <= buf: {:?}, buflen: {}, flags: {}",
        buf, buflen, flags
    );
    syscall_body!(sys_getrandom, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        if flags != 0 {
            return Err(LinuxError::EINVAL);
        }
        // fill the buffer 8 bytes at a time first, then fill the remaining bytes
        let buflen_mod = buflen % (core::mem::size_of::<i64>() / core::mem::size_of::<u8>());
        let buflen_div = buflen / (core::mem::size_of::<i64>() / core::mem::size_of::<u8>());
        for i in 0..buflen_div {
            *((buf as *mut u8 as *mut i64).add(i)) = sys_random() as i64;
        }
        for i in 0..buflen_mod {
            *((buf as *mut u8).add(buflen - buflen_mod + i)) = sys_rand() as u8;
        }
        Ok(buflen as ssize_t)
    })
}
