/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

/// No effect for "sp-naive" use.
pub fn init(_max_cpu_num: usize) {}

/// Always returns `0` for "sp-naive" use.
pub fn get_local_thread_pointer() -> usize {
    0
}

/// No effect for "sp-naive" use.
pub fn set_local_thread_pointer(_cpu_id: usize) {}
