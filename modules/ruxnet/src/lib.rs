/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

//! [Ruxos](https://github.com/syswonder/ruxos) network module.
//!
//! It provides unified networking primitives for TCP/UDP communication
//! using various underlying network stacks. Currently, only [smoltcp] is
//! supported.
//!
//! # Organization
//!
//! - [`TcpSocket`]: A TCP socket that provides POSIX-like APIs.
//! - [`UdpSocket`]: A UDP socket that provides POSIX-like APIs.
//! - [`dns_query`]: Function for DNS query.
//!
//! # Cargo Features
//!
//! - `smoltcp`: Use [smoltcp] as the underlying network stack. This is enabled
//!   by default.
//!
//! [smoltcp]: https://github.com/smoltcp-rs/smoltcp

#![no_std]
#![feature(c_variadic)]
#![feature(ip_in_core)]
#![feature(ip_bits)]
#![feature(new_uninit)]
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

#[macro_use]
extern crate log;
extern crate alloc;

mod unix;
pub use unix::{SocketAddrUnix, UnixSocket, UnixSocketType};

cfg_if::cfg_if! {
    if #[cfg(feature = "lwip")] {
        mod lwip_impl;
        use lwip_impl as net_impl;
        pub use lwip_impl::{IpAddr, Ipv4Addr, SocketAddr};
    }
    else if #[cfg(feature = "smoltcp")] {
        mod smoltcp_impl;
        use smoltcp_impl as net_impl;
        pub use self::net_impl::{bench_receive, bench_transmit};
    }
    else {
        error!("No network stack is selected");
    }
}

pub use self::net_impl::TcpSocket;
pub use self::net_impl::UdpSocket;
pub use self::net_impl::{dns_query, poll_interfaces};

use ruxdriver::{prelude::*, AxDeviceContainer};

/// Initializes the network subsystem by NIC devices.
pub fn init_network(mut net_devs: AxDeviceContainer<AxNetDevice>) {
    info!("Initialize network subsystem...");

    cfg_if::cfg_if! {
        if #[cfg(feature = "lwip")] {
            info!("  net stack: lwip");
        } else if #[cfg(feature = "smoltcp")] {
            info!("  net stack: smoltcp");
        } else {
            compile_error!("No network stack is selected");
        }
    }
    net_impl::init();
    unix::init_unix();
    while !net_devs.is_empty() {
        let dev = net_devs.take_one().expect("No NIC device found!");
        info!("  use NIC: {:?}", dev.device_name());
        net_impl::init_netdev(dev);
    }
}
