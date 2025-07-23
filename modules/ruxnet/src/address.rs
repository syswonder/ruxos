/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
//! Unified socket address
use core::net::{SocketAddrV4, SocketAddrV6};

use alloc::sync::Arc;
use axerrno::LinuxResult;
use axfs_vfs::AbsPath;
use ruxfs::fops;

use crate::{socket::Socket, socket_node::SocketNode};

#[derive(Debug, PartialEq, Eq, Clone)]
/// `Address` means the union of two `Addr`: UnixSocketAddr and Ipv4Addr
pub enum SocketAddress {
    /// Unix socket address
    Unix(UnixSocketAddr),
    /// ipv4 address
    Inet(SocketAddrV4),
    /// ipv6 address
    Inet6(SocketAddrV6),
}

impl From<core::net::SocketAddr> for SocketAddress {
    fn from(addr: core::net::SocketAddr) -> Self {
        match addr {
            core::net::SocketAddr::V4(socket_addr_v4) => SocketAddress::Inet(socket_addr_v4),
            core::net::SocketAddr::V6(socket_addr_v6) => SocketAddress::Inet6(socket_addr_v6),
        }
    }
}

impl From<SocketAddress> for core::net::SocketAddr {
    fn from(addr: SocketAddress) -> Self {
        match addr {
            SocketAddress::Unix(_) => {
                panic!("Cannot convert Unix socket address to core::net::SocketAddr")
            }
            SocketAddress::Inet(socket_addr_v4) => core::net::SocketAddr::V4(socket_addr_v4),
            SocketAddress::Inet6(socket_addr_v6) => core::net::SocketAddr::V6(socket_addr_v6),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
/// `UnixSocketAddr` represents the address of a UNIX domain socket.
/// see <https://www.man7.org/linux/man-pages/man7/unix.7.html>
pub enum UnixSocketAddr {
    /// A UNIX domain socket can be bound to a null-terminated filesystem pathname using `sys_bind`.
    PathName(AbsPath<'static>),
    /// A stream socket that has not been bound to a pathname using `sys_bind` has no name.
    /// Likewise, the two sockets created by `sys_socketpair` are unnamed. When the address
    /// of an unnamed socket is returned, its length is sizeof(`sa_family_t``), and `sun_path`
    /// should not be inspected.
    Unamed,
    /// An abstract socket address is distinguished (from a pathname socket) by the fact that
    /// `sun_path[0]` is a null byte ('\0'). The socket's address in this namespace is given by
    /// the additional bytes in sun_path that are covered by the specified length of the address
    /// structure. (Null bytes in the name have no special significance.)  The name has no
    /// connection with filesystem pathnames. When the address of an abstract socket is returned,
    /// the returned addrlen is greater than sizeof(sa_family_t) (i.e., greater than 2), and the
    /// name of the socket is contained in the first (addrlen - sizeof(sa_family_t)) bytes of sun_path.
    Abstract(Arc<[u8]>),
}

impl Default for UnixSocketAddr {
    fn default() -> Self {
        UnixSocketAddr::PathName(AbsPath::new(""))
    }
}

/// resolve unix socket addr by address
pub fn resolve_unix_socket_addr(address: &SocketAddress) -> LinuxResult<Arc<Socket>> {
    match address {
        SocketAddress::Unix(unix_socket_addr) => match unix_socket_addr {
            UnixSocketAddr::PathName(abs_path) => {
                let node = fops::lookup(abs_path)?;
                let socket_node = Arc::downcast::<SocketNode>(node.as_any_arc()).unwrap();
                Ok(socket_node.bound_socket())
            }
            UnixSocketAddr::Unamed => todo!(),
            UnixSocketAddr::Abstract(_) => todo!(),
        },
        _ => Err(axerrno::LinuxError::EINVAL),
    }
}
