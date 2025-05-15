/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
//! Combines both network and UNIX domain sockets under a common interface.
use core::net::SocketAddr;

use alloc::{sync::Arc, vec::Vec};
use axerrno::{LinuxError, LinuxResult};
use axfs_vfs::AbsPath;
use axio::PollState;
use axsync::Mutex;
use iovec::{IoVecsInput, IoVecsOutput};
use ruxfdtable::{FileLike, RuxStat};
use ruxfs::OpenFlags;

use crate::{
    address::SocketAddress,
    message::{ControlMessageData, MessageFlags, MessageReadInfo},
    unix::UnixSocket,
    ShutdownFlags, TcpSocket, UdpSocket,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Represents the type of socket communication semantics.
pub enum SocketType {
    /// Connection-oriented, reliable byte stream (SOCK_STREAM)
    /// Used by TCP and UNIX stream sockets
    Stream,
    /// Connectionless, unreliable datagrams (SOCK_DGRAM)
    /// Used by UDP and UNIX datagram sockets
    Datagram,
}

impl TryFrom<u32> for SocketType {
    type Error = LinuxError;

    fn try_from(ty: u32) -> Result<Self, Self::Error> {
        match ty {
            1 => Ok(SocketType::Stream),
            2 => Ok(SocketType::Datagram),
            _ => Err(LinuxError::EAFNOSUPPORT),
        }
    }
}

impl From<SocketType> for u32 {
    fn from(value: SocketType) -> Self {
        match value {
            SocketType::Stream => 1,
            SocketType::Datagram => 2,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
/// Socket domain
pub enum SocketDomain {
    /// UNIX domain sockets (AF_UNIX) - local IPC on same host
    /// Supports both stream and datagram types
    Unix,
    /// An AF_INET socket.
    Inet,
    /// An AF_INET6 socket.
    Inet6,
}

impl TryFrom<u16> for SocketDomain {
    type Error = LinuxError;

    fn try_from(domain: u16) -> Result<Self, Self::Error> {
        match domain {
            1 => Ok(SocketDomain::Unix),   // AF_UNIX
            2 => Ok(SocketDomain::Inet),   // AF_INET
            10 => Ok(SocketDomain::Inet6), // AF_INET6
            _ => Err(LinuxError::EAFNOSUPPORT),
        }
    }
}

/// Enum representing concrete socket implementations.
/// Combines both network and UNIX domain sockets under a common interface.
pub enum Socket {
    /// Tcp
    Tcp(Mutex<TcpSocket>),
    /// Udp
    Udp(Mutex<UdpSocket>),
    /// Unix
    Unix(UnixSocket),
}

impl FileLike for Socket {
    fn path(&self) -> AbsPath {
        AbsPath::new("/dev/socket")
    }

    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf, MessageFlags::empty())
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf, MessageFlags::empty())
    }

    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        Ok(RuxStat {
            st_mode: 0o140000 | 0o777u32, // S_IFSOCK | rwxrwxrwx;
            ..Default::default()
        })
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().poll()?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().poll()?),
            Socket::Unix(unixsocket) => unixsocket.poll(),
        }
    }

    fn set_flags(&self, flags: OpenFlags) -> LinuxResult {
        let nonblock = flags.contains(OpenFlags::O_NONBLOCK);
        match self {
            Socket::Udp(udpsocket) => udpsocket.lock().set_nonblocking(nonblock),
            Socket::Tcp(tcpsocket) => tcpsocket.lock().set_nonblocking(nonblock),
            Socket::Unix(unixsocket) => unixsocket.set_nonblocking(nonblock),
        }
        Ok(())
    }

    fn flags(&self) -> OpenFlags {
        let nonblock = match self {
            Socket::Udp(udpsocket) => udpsocket.lock().is_nonblocking(),
            Socket::Tcp(tcpsocket) => tcpsocket.lock().is_nonblocking(),
            Socket::Unix(unixsocket) => unixsocket.is_nonblocking(),
        };
        if nonblock {
            OpenFlags::O_NONBLOCK | OpenFlags::O_RDWR
        } else {
            OpenFlags::O_RDWR
        }
    }
}

impl Socket {
    /// Returns the address family/domain of the socket.
    pub fn domain(&self) -> SocketDomain {
        match self {
            Socket::Tcp(_) => SocketDomain::Inet,
            Socket::Udp(_) => SocketDomain::Inet,
            Socket::Unix(_) => SocketDomain::Unix,
        }
    }

    /// Returns the socket type
    pub fn socket_type(&self) -> SocketType {
        match self {
            Socket::Tcp(_) => SocketType::Stream,
            Socket::Udp(_) => SocketType::Datagram,
            Socket::Unix(unixsocket) => unixsocket.socket_type(),
        }
    }

    /// Returns a reference to the underlying UnixSocket if applicable.
    /// Panics if called on a network socket.
    pub fn as_unix_socket(&self) -> &UnixSocket {
        match self {
            Socket::Unix(unixsocket) => unixsocket,
            _ => panic!("Not a Unix socket"),
        }
    }

    /// Binds the socket to a specific address.
    /// For network sockets: binds to IP:port
    /// For UNIX sockets: binds to a filesystem path
    pub fn bind(self: Arc<Self>, address: SocketAddress) -> LinuxResult {
        match *self {
            Socket::Udp(ref udpsocket) => {
                if let SocketAddress::Inet(ipv4_addr) = address {
                    udpsocket
                        .lock()
                        .bind(SocketAddr::V4(ipv4_addr))
                        .map_err(LinuxError::from)
                } else {
                    Err(LinuxError::EINVAL)
                }
            }
            Socket::Tcp(ref tcpsocket) => {
                if let SocketAddress::Inet(ipv4_addr) = address {
                    tcpsocket
                        .lock()
                        .bind(SocketAddr::V4(ipv4_addr))
                        .map_err(LinuxError::from)
                } else {
                    Err(LinuxError::EINVAL)
                }
            }
            Socket::Unix(ref unixsocket) => unixsocket.bind(self.clone(), address),
        }
    }

    /// Starts listening for incoming connections (stream sockets only).
    /// backlog specifies the maximum pending connections queue size.
    pub fn listen(&self, backlog: i32) -> LinuxResult {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().listen()?),
            Socket::Unix(unixsocket) => Ok(unixsocket.listen(backlog)?),
        }
    }

    /// Accepts an incoming connection (stream sockets only).
    /// Returns a new Socket for the accepted connection.
    pub fn accept(&self) -> LinuxResult<Arc<Socket>> {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(Arc::new(Socket::Tcp(Mutex::new(
                tcpsocket.lock().accept()?,
            )))),
            Socket::Unix(unixsocket) => unixsocket.accept(),
        }
    }

    /// Connects to a remote endpoint.
    /// For datagram sockets, this sets the default destination.
    pub fn connect(self: Arc<Self>, address: SocketAddress) -> LinuxResult {
        match *self {
            Socket::Udp(ref udpsocket) => Ok(udpsocket.lock().connect(address.into())?),
            Socket::Tcp(ref tcpsocket) => Ok(tcpsocket.lock().connect(address.into())?),
            Socket::Unix(ref unixsocket) => unixsocket.connect(self.clone(), address),
        }
    }

    /// Returns the locally bound address of the socket.
    pub fn local_addr(&self) -> LinuxResult<SocketAddress> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().local_addr()?.into()),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().local_addr()?.into()),
            Socket::Unix(unixsocket) => unixsocket.local_addr(),
        }
    }

    /// Returns the address of the connected peer (if connected).
    pub fn peer_addr(&self) -> LinuxResult<SocketAddress> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().peer_addr()?.into()),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().peer_addr()?.into()),
            Socket::Unix(unixsocket) => unixsocket.peer_addr(),
        }
    }

    /// Sends data on a connected socket.
    pub fn send(&self, buf: &[u8], flags: MessageFlags) -> LinuxResult<usize> {
        self.sendmsg(
            &IoVecsInput::from_single_buffer(buf),
            None,
            &mut Vec::new(),
            flags,
        )
    }

    /// Sends data to a specific address (mainly for datagram sockets).
    pub fn sendto(
        &self,
        buf: &[u8],
        address: SocketAddress,
        flags: MessageFlags,
    ) -> LinuxResult<usize> {
        self.sendmsg(
            &IoVecsInput::from_single_buffer(buf),
            Some(address),
            &mut Vec::new(),
            flags,
        )
    }

    /// Advanced message sending with scatter/gather I/O and control messages.
    pub fn sendmsg(
        &self,
        iovecs: &IoVecsInput,
        address: Option<SocketAddress>,
        ancillary_data: &mut Vec<ControlMessageData>,
        flags: MessageFlags,
    ) -> LinuxResult<usize> {
        let mut bytes_send = 0;
        match self {
            Socket::Tcp(tcpsocket) => {
                let tcpsocket = tcpsocket.lock();
                if address.is_some() {
                    // The connection-mode socket was connected already but a recipient was specified.
                    return Err(LinuxError::EISCONN);
                } else {
                    for buf in iovecs.as_slices() {
                        bytes_send += tcpsocket.send(buf)?;
                    }
                }
            }
            Socket::Udp(udpsocket) => {
                let udpsocket = udpsocket.lock();
                if let Some(address) = address {
                    if let SocketAddress::Inet(ipv4_addr) = address {
                        for buf in iovecs.as_slices() {
                            bytes_send += udpsocket.send_to(buf, SocketAddr::V4(ipv4_addr))?;
                        }
                    } else {
                        return Err(LinuxError::EAFNOSUPPORT);
                    }
                } else {
                    for buf in iovecs.as_slices() {
                        bytes_send += udpsocket.send(buf)?;
                    }
                }
            }
            Socket::Unix(unixsocket) => {
                bytes_send += unixsocket.sendmsg(iovecs, address, ancillary_data, flags)?
            }
        }
        Ok(bytes_send)
    }

    /// Receives data from a connected socket.
    pub fn recv(&self, buf: &mut [u8], flags: MessageFlags) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().recv_from(buf).map(|e| e.0)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().recv(buf, flags)?),
            Socket::Unix(unixsocket) => {
                let info = unixsocket.recvmsg(&mut IoVecsOutput::from_single_buffer(buf), flags)?;
                Ok(info.bytes_read)
            }
        }
    }

    /// Receives data along with the sender's address (mainly for datagram sockets).
    pub fn recvfrom(
        &self,
        buf: &mut [u8],
        flags: MessageFlags,
    ) -> LinuxResult<(usize, Option<SocketAddress>)> {
        match self {
            // diff: must bind before recvfrom
            Socket::Udp(udpsocket) => {
                let (size, addr) = udpsocket.lock().recv_from(buf)?;
                Ok((size, Some(addr.into())))
            }
            Socket::Tcp(tcpsocket) => {
                let size = tcpsocket.lock().recv(buf, MessageFlags::empty())?;
                Ok((size, None))
            }
            Socket::Unix(unixsocket) => {
                let info = unixsocket.recvmsg(&mut IoVecsOutput::from_single_buffer(buf), flags)?;
                Ok((info.bytes_read, info.address))
            }
        }
    }

    /// Advanced message receiving with scatter/gather I/O and control messages
    pub fn recvmsg(
        &self,
        iovecs: &mut IoVecsOutput,
        flags: MessageFlags,
    ) -> LinuxResult<MessageReadInfo> {
        match self {
            Socket::Tcp(tcpsocket) => tcpsocket
                .lock()
                .recvmsg(iovecs, flags)
                .map_err(LinuxError::from),
            Socket::Udp(udpsocket) => udpsocket
                .lock()
                .recvmsg(iovecs, flags)
                .map_err(LinuxError::from),
            Socket::Unix(unixsocket) => unixsocket.recvmsg(iovecs, flags),
        }
    }

    /// Shuts down part or all of a full-duplex connection.
    pub fn shutdown(&self, how: ShutdownFlags) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                udpsocket.lock().peer_addr()?;
                udpsocket.lock().shutdown()?;
                Ok(())
            }

            Socket::Tcp(tcpsocket) => {
                tcpsocket.lock().peer_addr()?;
                tcpsocket.lock().shutdown()?;
                Ok(())
            }
            Socket::Unix(unixsocket) => unixsocket.shutdown(how),
        }
    }
}
