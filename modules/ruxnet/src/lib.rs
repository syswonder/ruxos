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
#![allow(incomplete_features)]
#![feature(inherent_associated_types)]

#[macro_use]
extern crate log;
extern crate alloc;

pub mod address;
pub mod message;
pub mod socket;
pub mod socket_node;
pub mod unix;

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

use axerrno::LinuxError;
use ruxdriver::{prelude::*, AxDeviceContainer};

bitflags::bitflags! {
    /// The flags for shutting down sockets.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct ShutdownFlags: i32 {
        /// Further transmissions will be disallowed.
        const WRITE = 1 << 0;
        /// Further receptions will be disallowed.
        const READ = 1 << 1;
    }
}

impl TryFrom<i32> for ShutdownFlags {
    type Error = LinuxError;

    fn try_from(how: i32) -> Result<Self, Self::Error> {
        const SHUT_RD: i32 = 0;
        const SHUT_WR: i32 = 1;
        const SHUT_RDWR: i32 = 2;
        match how {
            SHUT_RD => Ok(ShutdownFlags::READ),
            SHUT_WR => Ok(ShutdownFlags::WRITE),
            SHUT_RDWR => Ok(ShutdownFlags::READ | ShutdownFlags::WRITE),
            _ => Err(LinuxError::EINVAL),
        }
    }
}

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
    while !net_devs.is_empty() {
        let dev = net_devs.take_one().expect("No NIC device found!");
        info!("  use NIC: {:?}", dev.device_name());
        net_impl::init_netdev(dev);
    }
}
