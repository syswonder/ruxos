/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::{
    ffi::{c_int, c_uint},
    time::Duration,
};

use axerrno::{ax_err, ax_err_type, AxResult, LinuxError};
use bitflags::bitflags;
use ruxfutex::{futex_wait, futex_wait_bitset, futex_wake, futex_wake_bitset};

use crate::ctypes;

const FUTEX_OP_MASK: u32 = 0x0000_000F;
const FUTEX_FLAGS_MASK: u32 = u32::MAX ^ FUTEX_OP_MASK;

#[derive(PartialEq, Debug)]
#[repr(u32)]
#[allow(non_camel_case_types)]
pub enum FutexOp {
    FUTEX_WAIT = 0,
    FUTEX_WAKE = 1,
    FUTEX_FD = 2,
    FUTEX_REQUEUE = 3,
    FUTEX_CMP_REQUEUE = 4,
    FUTEX_WAKE_OP = 5,
    FUTEX_LOCK_PI = 6,
    FUTEX_UNLOCK_PI = 7,
    FUTEX_TRYLOCK_PI = 8,
    FUTEX_WAIT_BITSET = 9,
    FUTEX_WAKE_BITSET = 10,
}

bitflags! {
    pub struct FutexFlags : u32 {
        const FUTEX_PRIVATE         = 128;
        const FUTEX_CLOCK_REALTIME  = 256;
    }
}

impl FutexOp {
    pub fn from_u32(bits: u32) -> AxResult<FutexOp> {
        match bits {
            0 => Ok(FutexOp::FUTEX_WAIT),
            1 => Ok(FutexOp::FUTEX_WAKE),
            2 => Ok(FutexOp::FUTEX_FD),
            3 => Ok(FutexOp::FUTEX_REQUEUE),
            4 => Ok(FutexOp::FUTEX_CMP_REQUEUE),
            5 => Ok(FutexOp::FUTEX_WAKE_OP),
            6 => Ok(FutexOp::FUTEX_LOCK_PI),
            7 => Ok(FutexOp::FUTEX_UNLOCK_PI),
            8 => Ok(FutexOp::FUTEX_TRYLOCK_PI),
            9 => Ok(FutexOp::FUTEX_WAIT_BITSET),
            10 => Ok(FutexOp::FUTEX_WAKE_BITSET),
            _ => ax_err!(InvalidInput, "unknown futex op: {}", bits),
        }
    }
}

impl FutexFlags {
    pub fn from_u32(bits: u32) -> AxResult<FutexFlags> {
        FutexFlags::from_bits(bits)
            .ok_or_else(|| ax_err_type!(InvalidInput, "unknown futex flags: {}", bits))
    }
}

pub fn futex_op_and_flags_from_u32(bits: u32) -> AxResult<(FutexOp, FutexFlags)> {
    let op = {
        let op_bits = bits & FUTEX_OP_MASK;
        FutexOp::from_u32(op_bits)?
    };
    let flags = {
        let flags_bits = bits & FUTEX_FLAGS_MASK;
        FutexFlags::from_u32(flags_bits)?
    };
    Ok((op, flags))
}

/// `Futex` implementation inspired by occlum
pub fn sys_futex(
    uaddr: usize,
    op: c_uint,
    val: c_int,
    // timeout value, should be struct timespec pointer
    to: usize,
    // used by Requeue, unused for now
    #[allow(unused_variables)] uaddr2: c_int,
    // bitset
    val3: c_int,
) -> c_int {
    let futex_addr = uaddr as *const i32;
    let bitset = val3 as _;
    let max_count = val as _;
    let futex_val = val as _;

    syscall_body!(sys_futex, {
        let (op, _flag) = futex_op_and_flags_from_u32(op).map_err(LinuxError::from)?;
        let timeout = to as *const ctypes::timespec;
        let timeout = if !timeout.is_null()
            && matches!(op, FutexOp::FUTEX_WAIT | FutexOp::FUTEX_WAIT_BITSET)
        {
            let dur = unsafe { Duration::from(*timeout) };
            Some(dur)
        } else {
            None
        };
        debug!(
            "sys_futex <= addr: {:#x}, op: {:?}, val: {}, to: {:?}",
            uaddr, op, val, timeout,
        );

        let ret = match op {
            FutexOp::FUTEX_WAIT => futex_wait(futex_addr, futex_val, timeout).map(|_| 0),
            FutexOp::FUTEX_WAIT_BITSET => {
                futex_wait_bitset(futex_addr, futex_val, timeout, bitset).map(|_| 0)
            }
            FutexOp::FUTEX_WAKE => futex_wake(futex_addr, max_count),
            FutexOp::FUTEX_WAKE_BITSET => futex_wake_bitset(futex_addr, max_count, bitset),
            _ => ax_err!(Unsupported, "unsupported futex option: {:?}", op),
        };
        ret.map_err(LinuxError::from)
    })
}
