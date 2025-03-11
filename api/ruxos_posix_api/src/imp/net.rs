/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{sync::Arc, vec, vec::Vec};
use core::ffi::{c_char, c_int, c_void};
use core::iter;
use core::mem::size_of;
use core::net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4};
use ruxfs::{fops, AbsPath};

use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use axsync::Mutex;
use ruxfdtable::{FileLike, RuxStat};
use ruxnet::{SocketAddrUnix, TcpSocket, UdpSocket, UnixSocket, UnixSocketType};

use crate::ctypes;
use crate::imp::fs::flags_to_options;
use crate::utils::char_ptr_to_str;

fn addrun_convert(addr: *const ctypes::sockaddr_un) -> SocketAddrUnix {
    unsafe {
        SocketAddrUnix {
            sun_family: (*addr).sun_family,
            sun_path: (*addr).sun_path,
        }
    }
}

fn addrun_convert_withlen(addr: *const ctypes::sockaddr_un, addrlen: usize) -> SocketAddrUnix {
    unsafe {
        let sun_family = *(addr as *const u16);
        let mut sun_path_array = [0i8; 108];
        if addrlen > 2 {
            let len = (addrlen - 2).min(108);
            let src = (addr as *const u8).add(2) as *const i8;
            let dst = sun_path_array.as_mut_ptr();
            core::ptr::copy_nonoverlapping(src, dst, len);
        }
        SocketAddrUnix {
            sun_family,
            sun_path: sun_path_array,
        }
    }
}

#[derive(Debug)]
pub enum UnifiedSocketAddress {
    Net(SocketAddr),
    Unix(SocketAddrUnix),
}

pub enum Socket {
    Udp(Mutex<UdpSocket>),
    Tcp(Mutex<TcpSocket>),
    Unix(Mutex<UnixSocket>),
}

impl Socket {
    fn add_to_fd_table(self, flags: fops::OpenOptions) -> LinuxResult<c_int> {
        ruxtask::fs::add_file_like(Arc::new(self), flags)
    }

    fn from_fd(fd: c_int) -> LinuxResult<Arc<Self>> {
        let f = ruxtask::fs::get_file_like(fd)?;
        f.into_any()
            .downcast::<Self>()
            .map_err(|_| LinuxError::EINVAL)
    }

    fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().send(buf)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().send(buf)?),
            Socket::Unix(socket) => Ok(socket.lock().send(buf)?),
        }
    }

    fn recv(&self, buf: &mut [u8], flags: i32) -> LinuxResult<usize> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().recv_from(buf).map(|e| e.0)?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().recv(buf, flags)?),
            Socket::Unix(socket) => Ok(socket.lock().recv(buf, flags)?),
        }
    }

    pub fn poll(&self) -> LinuxResult<PollState> {
        match self {
            Socket::Udp(udpsocket) => Ok(udpsocket.lock().poll()?),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().poll()?),
            Socket::Unix(socket) => Ok(socket.lock().poll()?),
        }
    }

    fn local_addr(&self) -> LinuxResult<UnifiedSocketAddress> {
        match self {
            Socket::Udp(udpsocket) => Ok(UnifiedSocketAddress::Net(udpsocket.lock().local_addr()?)),
            Socket::Tcp(tcpsocket) => Ok(UnifiedSocketAddress::Net(tcpsocket.lock().local_addr()?)),
            Socket::Unix(unixsocket) => {
                Ok(UnifiedSocketAddress::Unix(unixsocket.lock().local_addr()?))
            }
        }
    }

    fn peer_addr(&self) -> LinuxResult<UnifiedSocketAddress> {
        match self {
            Socket::Udp(udpsocket) => Ok(UnifiedSocketAddress::Net(udpsocket.lock().peer_addr()?)),
            Socket::Tcp(tcpsocket) => Ok(UnifiedSocketAddress::Net(tcpsocket.lock().peer_addr()?)),
            Socket::Unix(unixsocket) => {
                Ok(UnifiedSocketAddress::Unix(unixsocket.lock().peer_addr()?))
            }
        }
    }

    fn bind(
        &self,
        socket_addr: *const ctypes::sockaddr,
        addrlen: ctypes::socklen_t,
    ) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                let addr = from_sockaddr(socket_addr, addrlen)?;
                Ok(udpsocket.lock().bind(addr)?)
            }
            Socket::Tcp(tcpsocket) => {
                let addr = from_sockaddr(socket_addr, addrlen)?;
                Ok(tcpsocket.lock().bind(addr)?)
            }
            Socket::Unix(unixsocket) => {
                if socket_addr.is_null() {
                    return Err(LinuxError::EFAULT);
                }
                Ok(unixsocket.lock().bind(addrun_convert_withlen(
                    socket_addr as *const ctypes::sockaddr_un,
                    addrlen.try_into().unwrap(),
                ))?)
            }
        }
    }

    fn connect(
        &self,
        socket_addr: *const ctypes::sockaddr,
        addrlen: ctypes::socklen_t,
    ) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                let addr = from_sockaddr(socket_addr, addrlen)?;
                Ok(udpsocket.lock().connect(addr)?)
            }
            Socket::Tcp(tcpsocket) => {
                let addr = from_sockaddr(socket_addr, addrlen)?;
                Ok(tcpsocket.lock().connect(addr)?)
            }
            Socket::Unix(socket) => {
                if socket_addr.is_null() {
                    return Err(LinuxError::EFAULT);
                }
                Ok(socket.lock().connect(addrun_convert_withlen(
                    socket_addr as *const ctypes::sockaddr_un,
                    addrlen.try_into().unwrap(),
                ))?)
            }
        }
    }

    fn sendto(
        &self,
        buf: &[u8],
        socket_addr: *const ctypes::sockaddr,
        addrlen: ctypes::socklen_t,
    ) -> LinuxResult<usize> {
        match self {
            // diff: must bind before sendto
            Socket::Udp(udpsocket) => {
                let addr = from_sockaddr(socket_addr, addrlen)?;
                Ok(udpsocket.lock().send_to(buf, addr)?)
            }
            Socket::Tcp(_) => Err(LinuxError::EISCONN),
            Socket::Unix(unixsocket) => {
                if socket_addr.is_null() {
                    return Err(LinuxError::EFAULT);
                }
                Ok(unixsocket.lock().sendto(
                    buf,
                    addrun_convert_withlen(
                        socket_addr as *const ctypes::sockaddr_un,
                        addrlen.try_into().unwrap(),
                    ),
                )?)
            }
        }
    }

    fn recvfrom(&self, buf: &mut [u8]) -> LinuxResult<(usize, Option<UnifiedSocketAddress>)> {
        match self {
            // diff: must bind before recvfrom
            Socket::Udp(udpsocket) => {
                let (size, addr) = udpsocket.lock().recv_from(buf)?;
                Ok((size, Some(UnifiedSocketAddress::Net(addr))))
            }
            Socket::Tcp(tcpsocket) => {
                let size = tcpsocket.lock().recv(buf, 0)?;
                Ok((size, None))
            }
            Socket::Unix(unixsocket) => {
                let guard = unixsocket.lock();
                match guard.get_sockettype() {
                    // diff: must bind before recvfrom
                    UnixSocketType::SockDgram => {
                        let (size, addr) = guard.recvfrom(buf)?;
                        Ok((size, addr.map(UnifiedSocketAddress::Unix)))
                    }
                    UnixSocketType::SockStream => {
                        let size = guard.recv(buf, 0)?;
                        Ok((size, None))
                    }
                    _ => Err(LinuxError::EOPNOTSUPP),
                }
            }
        }
    }

    fn listen(&self) -> LinuxResult {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(tcpsocket.lock().listen()?),
            Socket::Unix(socket) => Ok(socket.lock().listen()?),
        }
    }

    fn accept(&self) -> LinuxResult<Socket> {
        match self {
            Socket::Udp(_) => Err(LinuxError::EOPNOTSUPP),
            Socket::Tcp(tcpsocket) => Ok(Socket::Tcp(Mutex::new(tcpsocket.lock().accept()?))),
            Socket::Unix(unixsocket) => Ok(Socket::Unix(Mutex::new(unixsocket.lock().accept()?))),
        }
    }

    fn shutdown(&self) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => {
                let udpsocket = udpsocket.lock();
                udpsocket.peer_addr()?;
                udpsocket.shutdown()?;
                Ok(())
            }

            Socket::Tcp(tcpsocket) => {
                let tcpsocket = tcpsocket.lock();
                tcpsocket.peer_addr()?;
                tcpsocket.shutdown()?;
                Ok(())
            }
            Socket::Unix(socket) => {
                let socket = socket.lock();
                socket.peer_addr()?;
                socket.shutdown()?;
                Ok(())
            }
        }
    }
}

impl FileLike for Socket {
    fn path(&self) -> AbsPath {
        AbsPath::new("/dev/socket")
    }

    fn read(&self, buf: &mut [u8]) -> LinuxResult<usize> {
        self.recv(buf, 0)
    }

    fn write(&self, buf: &[u8]) -> LinuxResult<usize> {
        self.send(buf)
    }

    ///TODO
    fn flush(&self) -> LinuxResult {
        Ok(())
    }

    fn stat(&self) -> LinuxResult<RuxStat> {
        // not really implemented
        let st_mode = 0o140000 | 0o777u32; // S_IFSOCK | rwxrwxrwx
        Ok(RuxStat::from(ctypes::stat {
            st_ino: 1,
            st_nlink: 1,
            st_mode,
            st_uid: 1000,
            st_gid: 1000,
            st_blksize: 4096,
            ..Default::default()
        }))
    }

    fn into_any(self: Arc<Self>) -> Arc<dyn core::any::Any + Send + Sync> {
        self
    }

    fn poll(&self) -> LinuxResult<PollState> {
        self.poll()
    }

    fn set_nonblocking(&self, nonblock: bool) -> LinuxResult {
        match self {
            Socket::Udp(udpsocket) => udpsocket.lock().set_nonblocking(nonblock),
            Socket::Tcp(tcpsocket) => tcpsocket.lock().set_nonblocking(nonblock),
            Socket::Unix(unixsocket) => unixsocket.lock().set_nonblocking(nonblock),
        }
        Ok(())
    }
}

impl From<SocketAddrV4> for ctypes::sockaddr_in {
    fn from(addr: SocketAddrV4) -> ctypes::sockaddr_in {
        ctypes::sockaddr_in {
            sin_family: ctypes::AF_INET as u16,
            sin_port: addr.port().to_be(),
            sin_addr: ctypes::in_addr {
                // `s_addr` is stored as BE on all machines and the array is in BE order.
                // So the native endian conversion method is used so that it's never swapped.
                s_addr: u32::from_ne_bytes(addr.ip().octets()),
            },
            sin_zero: [0; 8],
        }
    }
}

impl From<SocketAddrUnix> for ctypes::sockaddr_un {
    fn from(addr: SocketAddrUnix) -> ctypes::sockaddr_un {
        ctypes::sockaddr_un {
            sun_family: addr.sun_family,
            sun_path: addr.sun_path,
        }
    }
}

impl From<ctypes::sockaddr_in> for SocketAddrV4 {
    fn from(addr: ctypes::sockaddr_in) -> SocketAddrV4 {
        SocketAddrV4::new(
            Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes()),
            u16::from_be(addr.sin_port),
        )
    }
}

fn unified_into_sockaddr(addr: UnifiedSocketAddress) -> (Vec<u8>, ctypes::socklen_t) {
    match addr {
        UnifiedSocketAddress::Net(addr) => {
            let (sockaddr, len) = in_into_sockaddr(addr);
            let bytes = unsafe {
                core::slice::from_raw_parts(&sockaddr as *const _ as *const u8, len as usize)
            }
            .to_vec();
            (bytes, len)
        }
        UnifiedSocketAddress::Unix(addr) => {
            let (sockaddr_un, len) = un_into_sockaddr(addr);
            let bytes = unsafe {
                core::slice::from_raw_parts(&sockaddr_un as *const _ as *const u8, len as usize)
            }
            .to_vec();
            (bytes, len)
        }
    }
}

fn un_into_sockaddr(addr: SocketAddrUnix) -> (ctypes::sockaddr_un, ctypes::socklen_t) {
    debug!(
        "convert unixsocket address {:?} into ctypes sockaddr_un",
        addr
    );
    let mut sockaddr_un = ctypes::sockaddr_un {
        sun_family: ctypes::AF_UNIX as u16,
        sun_path: [0; 108],
    };

    let path_bytes: Vec<c_char> = addr
        .sun_path
        .iter()
        .take_while(|&&c| c != 0)
        .chain(iter::once(&0))
        .map(|&c| c as c_char)
        .collect();

    let copy_len = path_bytes.len().min(sockaddr_un.sun_path.len());
    sockaddr_un.sun_path[..copy_len].copy_from_slice(&path_bytes[..copy_len]);

    let sun_path_offset = 2;
    let sockaddr_len = (sun_path_offset + copy_len) as ctypes::socklen_t;
    (sockaddr_un, sockaddr_len)
}

fn in_into_sockaddr(addr: SocketAddr) -> (ctypes::sockaddr, ctypes::socklen_t) {
    debug!("convert socket address {} into ctypes sockaddr", addr);
    match addr {
        SocketAddr::V4(addr) => (
            unsafe { *(&ctypes::sockaddr_in::from(addr) as *const _ as *const ctypes::sockaddr) },
            size_of::<ctypes::sockaddr>() as _,
        ),
        SocketAddr::V6(_) => panic!("IPv6 is not supported"),
    }
}

fn from_sockaddr(
    addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> LinuxResult<SocketAddr> {
    if addr.is_null() {
        return Err(LinuxError::EFAULT);
    }
    if addrlen != size_of::<ctypes::sockaddr>() as _ {
        return Err(LinuxError::EINVAL);
    }

    let mid = unsafe { *(addr as *const ctypes::sockaddr_in) };
    if mid.sin_family != ctypes::AF_INET as u16 {
        return Err(LinuxError::EINVAL);
    }

    let res = SocketAddr::V4(mid.into());
    debug!("    load sockaddr:{:#x} => {:?}", addr as usize, res);
    Ok(res)
}

/// Create an socket for communication.
///
/// Return the socket file descriptor.
pub fn sys_socket(domain: c_int, socktype: c_int, protocol: c_int) -> c_int {
    debug!("sys_socket <= {} {} {}", domain, socktype, protocol);
    let (domain, socktype, protocol) = (domain as u32, socktype as u32, protocol as u32);
    let fdflags = flags_to_options((socktype & ctypes::SOCK_CLOEXEC) as c_int, 0);
    let nonblock = (socktype & ctypes::SOCK_NONBLOCK) != 0;
    let socktype = socktype & !ctypes::SOCK_CLOEXEC & !ctypes::SOCK_NONBLOCK;
    syscall_body!(sys_socket, {
        match domain {
            ctypes::AF_INET => match (socktype, protocol) {
                (ctypes::SOCK_STREAM, ctypes::IPPROTO_TCP) | (ctypes::SOCK_STREAM, 0) => {
                    let tcp_socket = TcpSocket::new();
                    if nonblock {
                        tcp_socket.set_nonblocking(true);
                    }
                    Socket::Tcp(Mutex::new(tcp_socket)).add_to_fd_table(fdflags)
                }
                (ctypes::SOCK_DGRAM, ctypes::IPPROTO_UDP) | (ctypes::SOCK_DGRAM, 0) => {
                    Socket::Udp(Mutex::new(UdpSocket::new())).add_to_fd_table(fdflags)
                }
                _ => Err(LinuxError::EINVAL),
            },
            ctypes::AF_UNIX => match (socktype, protocol) {
                (ctypes::SOCK_STREAM, 0) => {
                    Socket::Unix(Mutex::new(UnixSocket::new(UnixSocketType::SockStream)))
                        .add_to_fd_table(fdflags)
                }
                (ctypes::SOCK_DGRAM, 0) => {
                    Socket::Unix(Mutex::new(UnixSocket::new(UnixSocketType::SockDgram)))
                        .add_to_fd_table(fdflags)
                }
                _ => Err(LinuxError::EINVAL),
            },
            _ => Err(LinuxError::EAFNOSUPPORT),
        }
    })
}

/// `setsockopt`, currently ignored
///
/// TODO: implement this
pub fn sys_setsockopt(
    fd: c_int,
    level: c_int,
    optname: c_int,
    _optval: *const c_void,
    optlen: ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_setsockopt <= fd: {}, level: {}, optname: {}, optlen: {}, IGNORED",
        fd, level, optname, optlen
    );
    syscall_body!(sys_setsockopt, Ok(0))
}

/// Bind a address to a socket.
///
/// Return 0 if success.
pub fn sys_bind(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_bind <= {} {:#x} {}",
        socket_fd, socket_addr as usize, addrlen
    );
    syscall_body!(sys_bind, {
        Socket::from_fd(socket_fd)?.bind(socket_addr, addrlen)?;
        Ok(0)
    })
}

/// Connects the socket to the address specified.
///
/// Return 0 if success.
pub fn sys_connect(
    socket_fd: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_connect <= {} {:#x} {}",
        socket_fd, socket_addr as usize, addrlen
    );
    syscall_body!(sys_connect, {
        Socket::from_fd(socket_fd)?.connect(socket_addr, addrlen)?;
        Ok(0)
    })
}

/// Send a message on a socket to the address specified.
///
/// Return the number of bytes sent if success.
pub fn sys_sendto(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_sendto <= {} {:#x} {} {} {:#x} {}",
        socket_fd, buf_ptr as usize, len, flag, socket_addr as usize, addrlen
    );
    if socket_addr.is_null() {
        debug!("sendto without address, use send instead");
        return sys_send(socket_fd, buf_ptr, len, flag);
    }
    syscall_body!(sys_sendto, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        Socket::from_fd(socket_fd)?.sendto(buf, socket_addr, addrlen)
    })
}

/// Send a message on a socket to the address connected.
///
/// Return the number of bytes sent if success.
pub fn sys_send(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!(
        "sys_send <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    syscall_body!(sys_send, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        Socket::from_fd(socket_fd)?.send(buf)
    })
}

/// Receive a message on a socket and get its source address.
///
/// Return the number of bytes received if success.
pub unsafe fn sys_recvfrom(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
    socket_addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_recvfrom <= {} {:#x} {} {} {:#x} {:#x}",
        socket_fd, buf_ptr as usize, len, flag, socket_addr as usize, addrlen as usize
    );
    if socket_addr.is_null() {
        debug!("recvfrom without address, use recv instead");
        return sys_recv(socket_fd, buf_ptr, len, flag);
    }

    syscall_body!(sys_recvfrom, {
        if buf_ptr.is_null() || addrlen.is_null() {
            warn!("recvfrom with null buffer or addrlen");
            return Err(LinuxError::EFAULT);
        }
        let socket = Socket::from_fd(socket_fd)?;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };

        let res = socket.recvfrom(buf)?;
        if let Some(addr) = res.1 {
            match addr {
                UnifiedSocketAddress::Net(addr) => unsafe {
                    (*socket_addr, *addrlen) = in_into_sockaddr(addr);
                },
                UnifiedSocketAddress::Unix(addr) => unsafe {
                    let sockaddr_un_size = addr.get_addr_len();
                    let sockaddr_un = SocketAddrUnix {
                        sun_family: 1u16, //  AF_UNIX
                        sun_path: addr.sun_path,
                    };
                    let original_addrlen = *addrlen as usize;
                    *addrlen = sockaddr_un_size as ctypes::socklen_t;
                    if original_addrlen < sockaddr_un_size {
                        warn!("Provided addr buf is too small, returned address will be truncated");
                        core::ptr::copy_nonoverlapping(
                            &sockaddr_un as *const SocketAddrUnix as *const u8,
                            socket_addr as *mut u8,
                            original_addrlen,
                        );
                    } else {
                        core::ptr::copy_nonoverlapping(
                            &sockaddr_un as *const SocketAddrUnix as *const u8,
                            socket_addr as *mut u8,
                            sockaddr_un_size,
                        );
                    }
                },
            }
        }
        Ok(res.0)
    })
}

/// Receive a message on a socket.
///
/// Return the number of bytes received if success.
pub fn sys_recv(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flag: c_int, // currently not used
) -> ctypes::ssize_t {
    debug!(
        "sys_recv <= {} {:#x} {} {}",
        socket_fd, buf_ptr as usize, len, flag
    );
    syscall_body!(sys_recv, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
        Socket::from_fd(socket_fd)?.recv(buf, flag)
    })
}

/// Listen for connections on a socket
///
/// Return 0 if success.
pub fn sys_listen(
    socket_fd: c_int,
    backlog: c_int, // currently not used
) -> c_int {
    debug!("sys_listen <= {} {}", socket_fd, backlog);
    syscall_body!(sys_listen, {
        Socket::from_fd(socket_fd)?.listen()?;
        Ok(0)
    })
}

/// Accept for connections on a socket
///
/// Return file descriptor for the accepted socket if success.
pub unsafe fn sys_accept(
    socket_fd: c_int,
    socket_addr: *mut ctypes::sockaddr,
    socket_len: *mut ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_accept <= {} {:#x} {:#x}",
        socket_fd, socket_addr as usize, socket_len as usize
    );
    syscall_body!(sys_accept, {
        if socket_addr.is_null() || socket_len.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let user_buf_len = unsafe { *socket_len } as usize;
        let socket = Socket::from_fd(socket_fd)?;
        let new_socket = socket.accept()?;
        let peer_addr = new_socket.peer_addr()?;
        let new_fd = Socket::add_to_fd_table(new_socket, fops::OpenOptions::new())?;

        let (addr_bytes, actual_len) = unified_into_sockaddr(peer_addr);

        let copy_len = user_buf_len.min(addr_bytes.len());

        unsafe {
            core::ptr::copy_nonoverlapping(addr_bytes.as_ptr(), socket_addr as *mut u8, copy_len);
            *socket_len = actual_len;
        }
        Ok(new_fd)
    })
}

/// Shut down a full-duplex connection.
///
/// Return 0 if success.
pub fn sys_shutdown(
    socket_fd: c_int,
    flag: c_int, // currently not used
) -> c_int {
    debug!("sys_shutdown <= {} {}", socket_fd, flag);
    syscall_body!(sys_shutdown, {
        Socket::from_fd(socket_fd)?.shutdown()?;
        Ok(0)
    })
}

/// Query addresses for a domain name.
///
/// Only IPv4. Ports are always 0. Ignore servname and hint.
/// Results' ai_flags and ai_canonname are 0 or NULL.
///
/// Return address number if success.
pub unsafe fn sys_getaddrinfo(
    nodename: *const c_char,
    servname: *const c_char,
    _hints: *const ctypes::addrinfo,
    res: *mut *mut ctypes::addrinfo,
) -> c_int {
    let name = char_ptr_to_str(nodename);
    let port = char_ptr_to_str(servname);
    debug!("sys_getaddrinfo <= {:?} {:?}", name, port);
    syscall_body!(sys_getaddrinfo, {
        if nodename.is_null() && servname.is_null() {
            return Ok(0);
        }
        if res.is_null() {
            return Err(LinuxError::EFAULT);
        }

        let port = port.map_or(0, |p| p.parse::<u16>().unwrap_or(0));
        let ip_addrs = if let Ok(domain) = name {
            if let Ok(a) = domain.parse::<IpAddr>() {
                vec![a]
            } else {
                ruxnet::dns_query(domain)?
            }
        } else {
            vec![Ipv4Addr::LOCALHOST.into()]
        };

        let len = ip_addrs.len().min(ctypes::MAXADDRS as usize);
        if len == 0 {
            return Ok(0);
        }

        let mut out: Vec<ctypes::aibuf> = Vec::with_capacity(len);
        for (i, &ip) in ip_addrs.iter().enumerate().take(len) {
            let buf = match ip {
                IpAddr::V4(ip) => ctypes::aibuf {
                    ai: ctypes::addrinfo {
                        ai_family: ctypes::AF_INET as _,
                        // TODO: This is a hard-code part, only return TCP parameters
                        ai_socktype: ctypes::SOCK_STREAM as _,
                        ai_protocol: ctypes::IPPROTO_TCP as _,
                        ai_addrlen: size_of::<ctypes::sockaddr_in>() as _,
                        ai_addr: core::ptr::null_mut(),
                        ai_canonname: core::ptr::null_mut(),
                        ai_next: core::ptr::null_mut(),
                        ai_flags: 0,
                    },
                    sa: ctypes::aibuf_sa {
                        sin: SocketAddrV4::new(ip, port).into(),
                    },
                    slot: i as i16,
                    lock: [0],
                    ref_: 0,
                },
                _ => panic!("IPv6 is not supported"),
            };
            out.push(buf);
            out[i].ai.ai_addr =
                unsafe { core::ptr::addr_of_mut!(out[i].sa.sin) as *mut ctypes::sockaddr };
            if i > 0 {
                out[i - 1].ai.ai_next = core::ptr::addr_of_mut!(out[i].ai);
            }
        }

        out[0].ref_ = len as i16;
        unsafe { *res = core::ptr::addr_of_mut!(out[0].ai) };
        core::mem::forget(out); // drop in `sys_freeaddrinfo`
        Ok(len)
    })
}

/// Free queried `addrinfo` struct
pub unsafe fn sys_freeaddrinfo(res: *mut ctypes::addrinfo) {
    if res.is_null() {
        return;
    }
    let aibuf_ptr = res as *mut ctypes::aibuf;
    let len = (*aibuf_ptr).ref_ as usize;
    assert!((*aibuf_ptr).slot == 0);
    assert!(len > 0);
    let vec = Vec::from_raw_parts(aibuf_ptr, len, len); // TODO: lock
    drop(vec);
}

/// Get current address to which the socket sockfd is bound.
pub unsafe fn sys_getsockname(
    sock_fd: c_int,
    addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_getsockname <= {} {:#x} {:#x}",
        sock_fd, addr as usize, addrlen as usize
    );
    syscall_body!(sys_getsockname, {
        if addr.is_null() || addrlen.is_null() {
            return Err(LinuxError::EFAULT);
        }

        let user_buf_len = unsafe { *addrlen } as usize;
        let (sockaddr_bytes, actual_len) = {
            let socket = Socket::from_fd(sock_fd)?;
            let local_addr = socket.local_addr()?;
            unified_into_sockaddr(local_addr)
        };

        let copy_len = user_buf_len.min(sockaddr_bytes.len());

        unsafe {
            core::ptr::copy_nonoverlapping(sockaddr_bytes.as_ptr(), addr as *mut u8, copy_len);
            *addrlen = actual_len;
        }
        Ok(0)
    })
}

/// get socket option
///
/// TODO: some options not impl, just return 0, like SO_RCVBUF SO_SNDBUF
pub fn sys_getsockopt(
    socket_fd: c_int,
    level: c_int,
    optname: c_int,
    optval: *mut c_void,
    optlen: *mut ctypes::socklen_t,
) -> c_int {
    unsafe {
        info!(
            "sys_getsockopt <= fd: {}, level: {}, optname: {}, optlen: {}, IGNORED",
            socket_fd,
            level,
            optname,
            core::ptr::read(optlen as *mut usize)
        );
    }
    syscall_body!(sys_getsockopt, {
        if optval.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let socket = Socket::from_fd(socket_fd)?;
        match level as u32 {
            ctypes::SOL_SOCKET => {
                let val = match optname as u32 {
                    ctypes::SO_ACCEPTCONN => match &*socket {
                        Socket::Udp(_) => 0,
                        Socket::Tcp(tcpsocket) => {
                            if tcpsocket.lock().is_listening() {
                                1
                            } else {
                                0
                            }
                        }
                        Socket::Unix(unixsocket) => {
                            if unixsocket.lock().is_listening() {
                                1
                            } else {
                                0
                            }
                        }
                    },
                    ctypes::SO_TYPE => match &*socket {
                        Socket::Udp(_) => ctypes::SOCK_DGRAM,
                        Socket::Tcp(_) => ctypes::SOCK_STREAM,
                        Socket::Unix(unixsocket) => match unixsocket.lock().get_sockettype() {
                            UnixSocketType::SockStream => ctypes::SOCK_STREAM,
                            UnixSocketType::SockDgram | UnixSocketType::SockSeqpacket => {
                                ctypes::SOCK_DGRAM
                            }
                        },
                    },
                    ctypes::SO_RCVLOWAT | ctypes::SO_SNDLOWAT | ctypes::SO_BROADCAST => 1,
                    ctypes::SO_ERROR
                    | ctypes::SO_DONTROUTE
                    | ctypes::SO_KEEPALIVE
                    | ctypes::SO_LINGER
                    | ctypes::SO_OOBINLINE
                    | ctypes::SO_RCVBUF
                    | ctypes::SO_RCVTIMEO
                    | ctypes::SO_REUSEADDR
                    | ctypes::SO_SNDBUF
                    | ctypes::SO_SNDTIMEO
                    | ctypes::SO_BINDTODEVICE => 0,
                    _ => return Err(LinuxError::ENOPROTOOPT),
                };

                unsafe {
                    core::ptr::write(optlen as *mut usize, core::mem::size_of::<i32>());
                    core::ptr::write(optval as *mut i32, val as i32);
                }

                Ok(0)
            }
            _ => Err(LinuxError::ENOSYS),
        }
    })
}

/// Get peer address to which the socket sockfd is connected.
pub unsafe fn sys_getpeername(
    sock_fd: c_int,
    socket_addr: *mut ctypes::sockaddr,
    socket_len: *mut ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_getpeername <= {} {:#x} {:#x}",
        sock_fd, socket_addr as usize, socket_len as usize
    );
    syscall_body!(sys_getpeername, {
        if socket_addr.is_null() || socket_len.is_null() {
            return Err(LinuxError::EFAULT);
        }
        if unsafe { *socket_len } < size_of::<ctypes::sockaddr>() as u32 {
            return Err(LinuxError::EINVAL);
        }
        let user_buf_len = unsafe { *socket_len } as usize;
        let sockaddr = Socket::from_fd(sock_fd)?.peer_addr()?;
        let (addr_bytes, actual_len) = unified_into_sockaddr(sockaddr);
        let copy_len = user_buf_len.min(addr_bytes.len());

        unsafe {
            core::ptr::copy_nonoverlapping(addr_bytes.as_ptr(), socket_addr as *mut u8, copy_len);
            *socket_len = actual_len;
        }

        if copy_len < addr_bytes.len() {
            Err(LinuxError::ENOBUFS)
        } else {
            Ok(0)
        }
    })
}

/// Send a message on a socket to the address connected.
/// The  message is pointed to by the elements of the array msg.msg_iov.
///
/// Return the number of bytes sent if success.
pub unsafe fn sys_sendmsg(
    socket_fd: c_int,
    msg: *const ctypes::msghdr,
    flags: c_int,
) -> ctypes::ssize_t {
    debug!("sys_sendmsg <= {} {:#x} {}", socket_fd, msg as usize, flags);
    syscall_body!(sys_sendmsg, {
        if msg.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let msg = *msg;
        if msg.msg_iov.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let iovs = core::slice::from_raw_parts(msg.msg_iov, msg.msg_iovlen as usize);
        let socket = Socket::from_fd(socket_fd)?;
        let mut ret = 0;

        for iov in iovs.iter() {
            if iov.iov_base.is_null() {
                return Err(LinuxError::EFAULT);
            }
            let buf = core::slice::from_raw_parts(iov.iov_base as *const u8, iov.iov_len);
            ret += match &socket as &Socket {
                Socket::Udp(udpsocket) => udpsocket.lock().send_to(
                    buf,
                    from_sockaddr(msg.msg_name as *const ctypes::sockaddr, msg.msg_namelen)?,
                )?,
                Socket::Tcp(tcpsocket) => tcpsocket.lock().send(buf)?,
                Socket::Unix(unixsocket) => unixsocket.lock().sendto(
                    buf,
                    addrun_convert(msg.msg_name as *const ctypes::sockaddr_un),
                )?,
            };
        }
        Ok(ret)
    })
}

/// Creates a pair of Unix domain sockets and stores the file descriptors in `sv`
///
/// This system call only works for UNIX domain sockets (AF_UNIX), which are used for communication
/// between processes on the same machine. It cannot be used for communication over the network (e.g.,
/// using AF_INET or AF_INET6). The created socket pair is anonymous, meaning it does not require
/// a pathname, and is typically used for communication between related processes (e.g., parent-child processes)
pub fn sys_socketpair(domain: c_int, socktype: c_int, protocol: c_int, sv: &mut [c_int]) -> c_int {
    info!("sys_socketpair <= domain: {domain}, socktype: {socktype}, protocol: {protocol}, sv pointer: {:#x}", sv.as_ptr() as usize);
    syscall_body!(sys_socketpair, {
        let (domain, socktype, _protocol) = (domain as u32, socktype as u32, protocol as u32);
        let fdflags = flags_to_options((socktype & ctypes::SOCK_CLOEXEC) as c_int, 0);
        let socktype = socktype & !ctypes::SOCK_CLOEXEC & !ctypes::SOCK_NONBLOCK;
        match domain {
            ctypes::AF_UNIX => {
                let (sk1, sk2) = match socktype {
                    ctypes::SOCK_STREAM => {
                        UnixSocket::create_socket_pair(UnixSocketType::SockStream)?
                    }
                    ctypes::SOCK_DGRAM => {
                        UnixSocket::create_socket_pair(UnixSocketType::SockDgram)?
                    }
                    _ => return Err(LinuxError::EAFNOSUPPORT),
                };
                sv[0] = Socket::Unix(Mutex::new(sk1)).add_to_fd_table(fdflags.clone())?;
                sv[1] = Socket::Unix(Mutex::new(sk2)).add_to_fd_table(fdflags)?;
                info!("create sv[0] {}, sv[1] {}", sv[0], sv[1]);
                Ok(0)
            }
            _ => return Err(LinuxError::EAFNOSUPPORT),
        }
    })
}
