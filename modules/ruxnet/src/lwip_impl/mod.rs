/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

#![allow(dead_code)]

mod addr;
mod dns;
mod driver;
mod tcp;
mod udp;

pub use self::addr::{IpAddr, Ipv4Addr, SocketAddr};
pub use self::dns::dns_query;
pub use self::driver::{init, init_netdev, poll_interfaces};
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;
use core::ffi::c_uint;
use ruxhal::time::current_time;

use axsync::Mutex;
use lazy_init::LazyInit;

static LWIP_MUTEX: LazyInit<Mutex<u32>> = LazyInit::new();

const RECV_QUEUE_LEN: usize = 16;
const ACCEPT_QUEUE_LEN: usize = 16;

#[no_mangle]
extern "C" fn sys_now() -> c_uint {
    current_time().as_millis() as c_uint
}
