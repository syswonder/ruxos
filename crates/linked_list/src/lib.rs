/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! Linked lists that supports arbitrary removal in constant time.
//!
//! It is based on the linked list implementation in [Rust-for-Linux][1].
//!
//! [1]: https://github.com/Rust-for-Linux/linux/blob/rust/rust/kernel/linked_list.rs

#![no_std]

mod linked_list;

pub mod unsafe_list;

pub use self::linked_list::{AdapterWrapped, List, Wrapper};
pub use unsafe_list::{Adapter, Cursor, Links};
