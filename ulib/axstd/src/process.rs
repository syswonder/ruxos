/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! A module for working with processes.
//!
//! Since ArceOS is a unikernel, there is no concept of processes. The
//! process-related functions will affect the entire system, such as [`exit`]
//! will shutdown the whole system.

/// Shutdown the whole system.
pub fn exit(_exit_code: i32) -> ! {
    arceos_api::sys::ax_terminate();
}
