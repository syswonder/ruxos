/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use core::time::Duration;

use axerrno::{AxError, AxResult};
use log::{debug, trace};

use super::{
    types::{FutexBucket, FutexKey},
    FUTEX_BUCKETS,
};

/// The bitset that matches any task,
/// used by [`futex_wake_bitset`] and [`futex_wait_bitset`].
pub const FUTEX_BITSET_MATCH_ANY: u32 = u32::MAX;

fn futex_wait_timeout(
    futex_addr: *const i32,
    futex_val: i32,
    timeout: Option<Duration>,
    bitset: u32,
    #[allow(unused_variables)] is_relative: bool,
) -> AxResult<()> {
    // Get the futex bucket
    let futex_key = FutexKey::new(futex_addr, bitset);
    let (_, futex_bucket) = FUTEX_BUCKETS.get_bucket(futex_key);

    let condition = || {
        // Check the futex value
        let actual_val = futex_key.load_val();
        trace!("futex_wait: expected {}, found {}", futex_val, actual_val);
        if actual_val != futex_val {
            // it's not actually an error but rather a notice to user,
            // so no `ax_err` and no warning
            return Err(AxError::WouldBlock);
        }

        Ok(())
    };

    // Lock the queue before checking futex value.
    match timeout {
        Some(timeout) => {
            #[cfg(feature = "irq")]
            let wait_timeout = if is_relative {
                FutexBucket::wait_timeout_meta_if
            } else {
                FutexBucket::wait_timeout_absolutely_meta_if
            };
            #[cfg(not(feature = "irq"))]
            let wait_timeout = FutexBucket::wait_timeout_absolutely_meta_if;
            let _is_timeout = wait_timeout(futex_bucket, timeout, futex_key, condition)?;
            Ok(())
        }
        None => futex_bucket.wait_meta_if(futex_key, condition),
    }
}

/// This operation tests that the value at the futex word
/// pointed to by the address `futex_addr` still contains the
/// expected value val, and if so, then sleeps waiting for a
/// [`futex_wake`] operation on the futex word. The load of the
/// value of the futex word is an atomic memory access (i.e.,
/// using atomic machine instructions of the respective
/// architecture). This load, the comparison with the
/// expected value, and starting to sleep are performed
/// atomically and totally ordered with respect to other futex
/// operations on the same futex word. If the thread starts
/// to sleep, it is considered a waiter on this futex word.
/// If the futex value does not match val, then the call fails
/// immediately with the error [`AxError::WouldBlock`].
///
/// The purpose of the comparison with the expected value is
/// to prevent lost wake-ups. If another thread changed the
/// value of the futex word after the calling thread decided
/// to block based on the prior value, and if the other thread
/// executed a [`futex_wake`] operation after
/// the value change and before this [`futex_wait`] operation,
/// then the calling thread will observe the value change and
/// will not start to sleep.
///
/// If the timeout is not [`None`], it specifies a timeout for the wait.
/// If timeout is NULL, the call blocks indefinitely.
///
/// Note that `timeout` is interpreted as a relative
/// value. This differs from other futex operations, where
/// timeout is interpreted as an absolute value. To obtain
/// the equivalent of [`futex_wait`] with an absolute timeout,
/// call [`futex_wait_bitset`] with `bitset` specified as
/// [`FUTEX_BITSET_MATCH_ANY`].
///
/// [`AxError::WouldBlock`]: axerrno::AxError::WouldBlock
pub fn futex_wait(
    futex_addr: *const i32,
    futex_val: i32,
    timeout: Option<Duration>,
) -> AxResult<()> {
    debug!(
        "futex_wait addr: {:#x}, val: {}, timeout: {:?}",
        futex_addr as usize, futex_val, timeout
    );
    futex_wait_timeout(futex_addr, futex_val, timeout, FUTEX_BITSET_MATCH_ANY, true)
}

/// This operation is like [`futex_wait`] except that `bitset`
/// is used to provide a 32-bit bit mask to the kernel. This bit
/// mask, in which at least one bit must be set, is stored in
/// the kernel-internal state of the waiter. See the
/// description of [`futex_wake_bitset`] for further details.
///
/// If timeout is not [`None`], it specifies an absolute timeout
/// for the wait operation. If timeout is [`None`], the operation can
/// block indefinitely.
pub fn futex_wait_bitset(
    futex_addr: *const i32,
    futex_val: i32,
    timeout: Option<Duration>,
    bitset: u32,
) -> AxResult<()> {
    debug!(
        "futex_wait_bitset addr: {:#x}, val: {}, timeout: {:?}, bitset: {:#x}",
        futex_addr as usize, futex_val, timeout, bitset
    );
    futex_wait_timeout(futex_addr, futex_val, timeout, bitset, false)
}

/// This operation wakes at most `max_count` of the waiters that are
/// waiting (e.g., inside [`futex_wait`]) on the futex word at the
/// address `futex_addr`. Most commonly, `max_count` is specified as either
/// 1 (wake up a single waiter) or [`usize::MAX`] (wake up all
/// waiters). No guarantee is provided about which waiters
/// are awoken (e.g., a waiter with a higher scheduling
/// priority is not guaranteed to be awoken in preference to a
/// waiter with a lower priority).
pub fn futex_wake(futex_addr: *const i32, max_count: usize) -> AxResult<usize> {
    futex_wake_bitset(futex_addr, max_count, FUTEX_BITSET_MATCH_ANY)
}

/// This operation is the same as [`futex_wake`] except that the
/// `btiset` argument is used to provide a 32-bit bit mask to the
/// kernel. This bit mask, in which at least one bit must be
/// set, is used to select which waiters should be woken up.
/// The selection is done by a bitwise AND of the "wake" bit
/// mask (i.e., the value in `bitset`) and the bit mask which is
/// stored in the kernel-internal state of the waiter (the
/// "wait" bit mask that is set using [`futex_wait_bitset`]). All
/// of the waiters for which the result of the AND is nonzero
/// are woken up; the remaining waiters are left sleeping.
///
/// The effect of [`futex_wait_bitset`] and [`futex_wake_bitset`] is
/// to allow selective wake-ups among multiple waiters that
/// are blocked on the same futex. However, note that,
/// depending on the use case, employing this bit-mask
/// multiplexing feature on a futex can be less efficient than
/// simply using multiple futexes, because employing bit-mask
/// multiplexing requires the kernel to check all waiters on a
/// futex, including those that are not interested in being
/// woken up (i.e., they do not have the relevant bit set in
/// their "wait" bit mask). In the current implementation, this does
/// not make a significant difference.
///
/// The constant [`FUTEX_BITSET_MATCH_ANY`], which corresponds to
/// all 32 bits set in the bit mask, can be used as the `bitset`
/// argument for [`futex_wait_bitset`] and [`futex_wake_bitset`].
/// Other than differences in the handling of the timeout
/// argument, the [`futex_wait`] operation is equivalent to
/// [`futex_wait_bitset`] with `bitset` specified as
/// [`FUTEX_BITSET_MATCH_ANY`]; that is, allow a wake-up by any
/// waker. The [`futex_wake`] operation is equivalent to
/// [`futex_wake_bitset`] with `bitset` specified as
/// [`FUTEX_BITSET_MATCH_ANY`]; that is, wake up any waiter(s).
pub fn futex_wake_bitset(futex_addr: *const i32, max_count: usize, bitset: u32) -> AxResult<usize> {
    debug!(
        "futex_wake_bitset addr: {:#x}, max_count: {}, bitset: {:#x}",
        futex_addr as usize, max_count, bitset
    );

    let futex_key = FutexKey::new(futex_addr, bitset);
    let (_, futex_bucket) = FUTEX_BUCKETS.get_bucket(futex_key);

    let mut count = 0;

    // Wake up the tasks in the bucket
    let task_count = futex_bucket.notify_task_if(false, |task, &key| {
        trace!(
            "futex wake: count: {}, key: {:?}, futex_key: {:?}, bitset: {}, is_notified: {}, task: {:?}",
            count,
            key,
            futex_key,
            bitset,
            !(count >= max_count || futex_key != key || (bitset & key.bitset()) == 0),
            task,
        );
        if !task.is_blocked() {
            return true;
        }
        if count >= max_count || futex_key != key || (bitset & key.bitset()) == 0 {
            false
        } else {
            count += 1;
            true
        }
    });
    Ok(task_count)
}
