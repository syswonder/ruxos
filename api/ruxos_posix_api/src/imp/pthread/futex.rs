/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::collections::{BTreeMap, VecDeque};
use core::{ffi::c_int, time::Duration};

use axerrno::LinuxError;
use axsync::Mutex;
use memory_addr::VirtAddr;
use ruxtask::{current, AxTaskRef, TaskState, WaitQueue};

use crate::ctypes;

enum FutexFlags {
    Wait,
    Wake,
    Requeue,
    Unsupported,
}

impl FutexFlags {
    pub fn from(val: c_int) -> Self {
        match val & 0x7f {
            0 => FutexFlags::Wait,
            1 => FutexFlags::Wake,
            3 => FutexFlags::Requeue,
            _ => FutexFlags::Unsupported,
        }
    }
}

pub static FUTEX_WAIT_TASK: Mutex<BTreeMap<VirtAddr, VecDeque<(AxTaskRef, c_int)>>> =
    Mutex::new(BTreeMap::new());

pub static WAIT_FOR_FUTEX: WaitQueue = WaitQueue::new();

/// `Futex` implementation inspired by Starry
pub fn sys_futex(
    uaddr: usize,
    op: c_int,
    val: c_int,
    // timeout value, should be struct timespec pointer
    to: usize,
    // used by Requeue
    _uaddr2: c_int,
    // not supported
    _val3: c_int,
) -> c_int {
    debug!(
        "sys_futex <= addr: {:#x}, op: {}, val: {}, to: {}",
        uaddr, op, val, to
    );
    check_dead_wait();
    let flag = FutexFlags::from(op);
    let current_task = current();
    let timeout = if to != 0 {
        let dur = unsafe { Duration::from(*(to as *const ctypes::timespec)) };
        dur.as_nanos() as u64
    } else {
        0
    };
    syscall_body!(sys_futex, {
        match flag {
            FutexFlags::Wait => {
                let real_futex_val = unsafe { (uaddr as *const c_int).read_volatile() };
                trace!("real_futex_val: {}, expect: {}", real_futex_val, val);
                if real_futex_val != val {
                    return Err(LinuxError::EAGAIN);
                }
                let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
                let wait_list = if let alloc::collections::btree_map::Entry::Vacant(e) =
                    futex_wait_task.entry(uaddr.into())
                {
                    e.insert(VecDeque::new());
                    futex_wait_task.get_mut(&(uaddr.into())).unwrap()
                } else {
                    futex_wait_task.get_mut(&(uaddr.into())).unwrap()
                };

                let next = current_task.as_task_ref().clone();
                wait_list.push_back((next, val));
                drop(futex_wait_task);

                // TODO: check signals
                if timeout == 0 {
                    ruxtask::yield_now();
                } else {
                    #[cfg(feature = "irq")]
                    {
                        let timeout = WAIT_FOR_FUTEX.wait_timeout(Duration::from_nanos(timeout));
                        if !timeout {
                            // TODO: should check signals
                            return Err(LinuxError::EINTR);
                        }
                    }
                }
                Ok(0)
            }
            FutexFlags::Wake => {
                trace!(
                    "thread id: {}, wake addr: {:#x}",
                    current_task.id().as_u64(),
                    uaddr
                );
                let mut futex_wait_task = FUTEX_WAIT_TASK.lock();
                if futex_wait_task.contains_key(&(uaddr.into())) {
                    let wait_list = futex_wait_task.get_mut(&(uaddr.into())).unwrap();
                    loop {
                        if let Some((task, _)) = wait_list.pop_front() {
                            // wake up a waiting task
                            if !task.is_blocked() {
                                continue;
                            }
                            trace!("Wake task: {}", task.id().as_u64());
                            drop(futex_wait_task);
                            WAIT_FOR_FUTEX.notify_task(false, &task);
                        } else {
                            drop(futex_wait_task);
                        }
                        break;
                    }
                } else {
                    drop(futex_wait_task);
                }
                ruxtask::yield_now();
                Ok(val)
            }
            FutexFlags::Requeue => {
                debug!("unimplemented for REQUEUE");
                Ok(0)
            }
            _ => Err(LinuxError::EFAULT),
        }
    })
}

fn check_dead_wait() {
    let mut futex_wait_tast = FUTEX_WAIT_TASK.lock();
    for (vaddr, wait_list) in futex_wait_tast.iter_mut() {
        let real_futex_val = unsafe { ((*vaddr).as_usize() as *const u32).read_volatile() };
        for (task, val) in wait_list.iter() {
            if real_futex_val as i32 != *val && task.state() == TaskState::Blocked {
                WAIT_FOR_FUTEX.notify_task(false, task);
            }
        }
        wait_list.retain(|(task, val)| {
            real_futex_val as i32 == *val && task.state() == TaskState::Blocked
        });
    }
}
