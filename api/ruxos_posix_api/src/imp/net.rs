/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use alloc::{sync::Arc, vec, vec::Vec};
use axsync::Mutex;
use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::net::{IpAddr, Ipv4Addr, SocketAddrV4};
use iovec::{read_iovecs_ptr, IoVecsInput, IoVecsOutput};
use ruxnet::address::{SocketAddress, UnixSocketAddr};
use ruxnet::message::{ControlMessageData, MessageFlags};
use ruxnet::socket::{Socket, SocketDomain, SocketType};
use ruxnet::unix::UnixSocket;
use ruxtask::fs::{add_file_like, get_file_like};

use axerrno::{LinuxError, LinuxResult};
use ruxfdtable::OpenFlags;
use ruxnet::{ShutdownFlags, TcpSocket, UdpSocket};

use crate::ctypes::{self};
use crate::imp::fs::parse_path;
use crate::utils::char_ptr_to_str;

const SA_FAMILY_SIZE: usize = size_of::<ctypes::sa_family_t>();

fn parse_socket_address(
    addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> LinuxResult<SocketAddress> {
    let domain = SocketDomain::try_from(unsafe { *(addr as *const u16) })?;
    match domain {
        SocketDomain::Inet => {
            if addrlen < (size_of::<ctypes::sockaddr_in>() as u32) {
                return Err(LinuxError::EINVAL);
            }
            let addr = unsafe { *(addr as *const ctypes::sockaddr_in) };
            Ok(SocketAddress::Inet(SocketAddrV4::new(
                Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes()),
                u16::from_be(addr.sin_port),
            )))
        }
        SocketDomain::Unix => {
            const UNIX_PATH_MAX: u32 = 108;
            if addrlen < 2 || addrlen > UNIX_PATH_MAX + SA_FAMILY_SIZE as u32 {
                return Err(LinuxError::EINVAL);
            }
            let len = (addrlen - 2).min(UNIX_PATH_MAX);
            let sun_path = unsafe {
                core::slice::from_raw_parts((addr as *const u8).add(SA_FAMILY_SIZE), len as usize)
            };
            if sun_path[0] == b'\0' {
                let path = sun_path[1..]
                    .iter()
                    .take_while(|&&c| c != 0)
                    .cloned()
                    .collect::<Vec<_>>();
                Ok(SocketAddress::Unix(UnixSocketAddr::Abstract(Arc::from(
                    path,
                ))))
            } else {
                let abs_path = parse_path(sun_path.as_ptr() as *const c_char)?;
                Ok(SocketAddress::Unix(UnixSocketAddr::PathName(abs_path)))
            }
        }
        SocketDomain::Inet6 => Err(LinuxError::EAFNOSUPPORT),
    }
}

fn write_sockaddr_with_max_len(
    address: SocketAddress,
    addr_ptr: *mut ::core::ffi::c_void,
    max_len: u32,
) -> LinuxResult<u32> {
    if addr_ptr.is_null() {
        warn!("write_sockaddr_with_max_len with null pointer");
        return Err(LinuxError::EFAULT);
    }
    let actual_len = match address {
        SocketAddress::Unix(unix_addr) => match unix_addr {
            UnixSocketAddr::PathName(abs_path) => {
                let actual_len = SA_FAMILY_SIZE + abs_path.len() + 1;
                let write_len = core::cmp::min(actual_len, max_len as usize);
                unsafe { *(addr_ptr as *mut u16) = ctypes::AF_UNIX as u16 };
                let sun_path_ptr = unsafe { addr_ptr.add(SA_FAMILY_SIZE) };
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        abs_path.as_ptr(),
                        sun_path_ptr as *mut u8,
                        write_len,
                    )
                };
                unsafe { *(addr_ptr as *mut u8).add(write_len - 1) = b'\0' }; // Null-terminate the path
                actual_len
            }
            UnixSocketAddr::Unamed => {
                warn!("write unamed unix addr");
                let actual_len = SA_FAMILY_SIZE + 1;
                unsafe { *(addr_ptr as *mut u16) = ctypes::AF_UNIX as u16 };
                let sun_path_ptr = unsafe { addr_ptr.add(SA_FAMILY_SIZE) };
                unsafe { *(sun_path_ptr as *mut u8) = b'\0' };
                actual_len
            }
            UnixSocketAddr::Abstract(_) => todo!(),
        },
        SocketAddress::Inet(ipv4_addr) => {
            let actual_len = size_of::<ctypes::sockaddr_in>();
            let write_len = core::cmp::min(actual_len, max_len as usize);
            let sockaddr_in = ctypes::sockaddr_in::from(ipv4_addr);
            unsafe {
                core::ptr::copy_nonoverlapping(
                    &sockaddr_in as *const ctypes::sockaddr_in as *const u8,
                    addr_ptr as *mut u8,
                    write_len,
                );
            }
            actual_len
        }
        SocketAddress::Inet6(_) => todo!(),
    };
    Ok(actual_len as u32)
}

fn write_sockaddr_with_max_len_ptr(
    address: SocketAddress,
    addr_ptr: *mut ::core::ffi::c_void,
    max_len_ptr: *mut ctypes::socklen_t,
) -> LinuxResult {
    if addr_ptr.is_null() || max_len_ptr.is_null() {
        warn!("write_sockaddr_with_max_len_ptr with null pointer");
        return Err(LinuxError::EFAULT);
    }
    let actual_len = write_sockaddr_with_max_len(address, addr_ptr, unsafe { *max_len_ptr })?;
    unsafe { *max_len_ptr = actual_len as ctypes::socklen_t };
    Ok(())
}

impl From<ctypes::sockaddr_in> for SocketAddrV4 {
    fn from(addr: ctypes::sockaddr_in) -> SocketAddrV4 {
        SocketAddrV4::new(
            Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes()),
            u16::from_be(addr.sin_port),
        )
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

/// Create an socket for communication.
///
/// Return the socket file descriptor.
pub fn sys_socket(domain: c_int, socktype: c_int, protocol: c_int) -> c_int {
    syscall_body!(sys_socket, {
        let socktype = socktype as u32;
        let mut flags = OpenFlags::empty();
        let nonblock = socktype & ctypes::SOCK_NONBLOCK != 0;
        if socktype & ctypes::SOCK_CLOEXEC != 0 {
            flags |= OpenFlags::O_CLOEXEC;
        }
        let socktype = SocketType::try_from(socktype & 0xf)?;
        let domain = SocketDomain::try_from(domain as u16)?;
        debug!("sys_socket <= {:?} {:?} {}", domain, socktype, protocol);
        debug!("nonblock: {nonblock}, cloexec: {flags:?}");
        let f = match domain {
            SocketDomain::Inet => match socktype {
                SocketType::Stream => Arc::new(Socket::Tcp(Mutex::new(TcpSocket::new(nonblock)))),
                SocketType::Datagram => Arc::new(Socket::Udp(Mutex::new(UdpSocket::new()))),
            },
            SocketDomain::Unix => UnixSocket::create_socket(socktype, nonblock),
            SocketDomain::Inet6 => return Err(LinuxError::EAFNOSUPPORT),
        };
        add_file_like(f, flags)
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
    syscall_body!(sys_bind, {
        let address = parse_socket_address(socket_addr, addrlen)?;
        debug!("sys_bind <= {socket_fd} {address:?}");
        socket_from_fd(socket_fd)?.bind(address)?;
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
        let address = parse_socket_address(socket_addr, addrlen)?;
        socket_from_fd(socket_fd)?.connect(address)?;
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
    flags: c_int,
    socket_addr: *const ctypes::sockaddr,
    addrlen: ctypes::socklen_t,
) -> ctypes::ssize_t {
    if socket_addr.is_null() {
        debug!("sendto without address, use send instead");
        return sys_send(socket_fd, buf_ptr, len, flags);
    }
    syscall_body!(sys_sendto, {
        let address = parse_socket_address(socket_addr, addrlen)?;
        let flags = MessageFlags::from_bits_truncate(flags);
        debug!(
            "sys_sendto <= {socket_fd} {:#x} {len} {flags:?} {address:?}",
            buf_ptr as usize
        );
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        socket_from_fd(socket_fd)?.sendto(buf, address, flags)
    })
}

/// Send a message on a socket to the address connected.
///
/// Return the number of bytes sent if success.
pub fn sys_send(
    socket_fd: c_int,
    buf_ptr: *const c_void,
    len: ctypes::size_t,
    flag: c_int,
) -> ctypes::ssize_t {
    syscall_body!(sys_send, {
        let flags = MessageFlags::from_bits_truncate(flag);
        debug!(
            "sys_send <= {socket_fd} {:#x} {len} {flags:?}",
            buf_ptr as usize
        );
        let buf = unsafe { core::slice::from_raw_parts(buf_ptr as *const u8, len) };
        socket_from_fd(socket_fd)?.send(buf, flags)
    })
}

/// Receive a message on a socket and get its source address.
///
/// Return the number of bytes received if success.
pub unsafe fn sys_recvfrom(
    socket_fd: c_int,
    buf_ptr: *mut c_void,
    len: ctypes::size_t,
    flags: c_int,
    socket_addr: *mut ctypes::sockaddr,
    addrlen: *mut ctypes::socklen_t,
) -> ctypes::ssize_t {
    if socket_addr.is_null() {
        debug!("recvfrom without address, use recv instead");
        return sys_recv(socket_fd, buf_ptr, len, flags);
    }

    syscall_body!(sys_recvfrom, {
        let flags = MessageFlags::from_bits_truncate(flags);
        debug!(
            "sys_recvfrom <= {socket_fd} {:#x} {len} {flags:?} {:#x} {:#x}",
            buf_ptr as usize, socket_addr as usize, addrlen as usize
        );
        if buf_ptr.is_null() || addrlen.is_null() {
            warn!("recvfrom with null buffer or addrlen");
            return Err(LinuxError::EFAULT);
        }
        let socket = socket_from_fd(socket_fd)?;
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };

        let res = socket.recvfrom(buf, flags)?;
        if let Some(addr) = res.1 {
            write_sockaddr_with_max_len_ptr(addr, socket_addr as _, addrlen)?;
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
    flags: c_int,
) -> ctypes::ssize_t {
    syscall_body!(sys_recv, {
        if buf_ptr.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let buf = unsafe { core::slice::from_raw_parts_mut(buf_ptr as *mut u8, len) };
        let flags = MessageFlags::from_bits_truncate(flags);
        debug!(
            "sys_recv <= {} {:#x} {} {:?}",
            socket_fd, buf_ptr as usize, len, flags
        );
        socket_from_fd(socket_fd)?.recv(buf, flags)
    })
}

/// Listen for connections on a socket
///
/// Return 0 if success.
pub fn sys_listen(socket_fd: c_int, backlog: c_int) -> c_int {
    debug!("sys_listen <= {} {}", socket_fd, backlog);
    syscall_body!(sys_listen, {
        socket_from_fd(socket_fd)?.listen(backlog)?;
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
        let socket = socket_from_fd(socket_fd)?;
        let new_socket = socket.accept()?;
        if !socket_addr.is_null() {
            let peer_addr = new_socket.peer_addr()?;
            write_sockaddr_with_max_len_ptr(peer_addr, socket_addr as _, socket_len)?;
        }
        let new_fd = add_file_like(new_socket, OpenFlags::empty())?;
        Ok(new_fd)
    })
}

/// Shut down a full-duplex connection.
///
/// Return 0 if success.
pub fn sys_shutdown(socket_fd: c_int, how: c_int) -> c_int {
    syscall_body!(sys_shutdown, {
        let flags = ShutdownFlags::try_from(how)?;
        debug!("sys_shutdown <= {} {:?}", socket_fd, flags);
        socket_from_fd(socket_fd)?.shutdown(flags)?;
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
        let local_addr = socket_from_fd(sock_fd)?.local_addr()?;
        debug!("socket {sock_fd} local address: {local_addr:?}");
        write_sockaddr_with_max_len_ptr(local_addr, addr as _, addrlen)?;
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
        let socket = socket_from_fd(socket_fd)?;
        match level as u32 {
            ctypes::SOL_SOCKET => {
                let val = match optname as u32 {
                    ctypes::SO_ACCEPTCONN => match &*socket {
                        Socket::Udp(_) => 0,
                        Socket::Tcp(tcpsocket) => tcpsocket.lock().is_listening() as u32,
                        Socket::Unix(unixsocket) => unixsocket.is_listening() as u32,
                    },
                    ctypes::SO_TYPE => match &*socket {
                        Socket::Udp(_) => ctypes::SOCK_DGRAM,
                        Socket::Tcp(_) => ctypes::SOCK_STREAM,
                        Socket::Unix(unixsocket) => unixsocket.socket_type().into(),
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
pub fn sys_getpeername(
    sock_fd: c_int,
    socket_addr: *mut ctypes::sockaddr,
    socket_len: *mut ctypes::socklen_t,
) -> c_int {
    debug!(
        "sys_getpeername <= {} {:#x} {:#x}",
        sock_fd, socket_addr as usize, socket_len as usize
    );
    syscall_body!(sys_getpeername, {
        let peer_addr = socket_from_fd(sock_fd)?.peer_addr()?;
        debug!("socket {sock_fd} peer address: {peer_addr:?}");
        write_sockaddr_with_max_len_ptr(peer_addr, socket_addr as _, socket_len)?;
        Ok(0)
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
        let msghdr = unsafe { *msg };
        if msghdr.msg_iov.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let dst_address = if msghdr.msg_name.is_null() {
            None
        } else {
            Some(parse_socket_address(
                msghdr.msg_name as _,
                msghdr.msg_namelen,
            )?)
        };

        let flags = MessageFlags::from_bits_truncate(flags);
        let socket = socket_from_fd(socket_fd)?;
        debug!("send msg to {dst_address:?} with flags {flags:?}");

        let mut ancillary_data = Vec::new();
        let cmsg_header_size = size_of::<ctypes::cmsghdr>();
        let mut cmsg_header_ptr = msghdr.msg_control as *const u8;
        let cmsg_header_ptr_end = unsafe { cmsg_header_ptr.add(msghdr.msg_controllen as usize) };
        if !msghdr.msg_control.is_null() {
            loop {
                if cmsg_header_ptr >= cmsg_header_ptr_end {
                    break;
                }
                let cmsg_header = unsafe { *(cmsg_header_ptr as *const ctypes::cmsghdr) };
                if (cmsg_header.cmsg_len as usize) < cmsg_header_size {
                    return Err(LinuxError::EINVAL);
                }
                let cmsg_data_ptr = unsafe { cmsg_header_ptr.add(cmsg_header_size) };
                let cmsg_data_size = (cmsg_header.cmsg_len) as usize - cmsg_header_size;
                let cmsg_data =
                    unsafe { core::slice::from_raw_parts(cmsg_data_ptr, cmsg_data_size).to_vec() };
                ancillary_data.push(ControlMessageData::try_new(
                    cmsg_header.cmsg_level,
                    cmsg_header.cmsg_type,
                    cmsg_data,
                )?);
                cmsg_header_ptr = unsafe { cmsg_header_ptr.add(cmsg_header.cmsg_len as usize) };
                cmsg_header_ptr = cmsg_align(cmsg_header_ptr as _) as *const u8;
            }
        }
        let iovecs =
            IoVecsInput::from_iovecs(read_iovecs_ptr(msghdr.msg_iov as _, msghdr.msg_iovlen as _));
        let bytes_send = socket.sendmsg(&iovecs, dst_address, &mut ancillary_data, flags)?;
        Ok(bytes_send)
    })
}

/// Receives a message from a socket, supporting scatter/gather I/O and ancillary data.
pub unsafe fn sys_recvmsg(
    socket_fd: c_int,
    msg: *mut ctypes::msghdr,
    flags: c_int,
) -> ctypes::ssize_t {
    debug!("sys_recvmsg <= {} {:#x} {}", socket_fd, msg as usize, flags);
    syscall_body!(sys_recvmsg, {
        if msg.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let mut msghdr = unsafe { *msg };
        if msghdr.msg_iov.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let flags = MessageFlags::from_bits_truncate(flags);
        let socket = socket_from_fd(socket_fd)?;
        let mut iovecs =
            IoVecsOutput::from_iovecs(read_iovecs_ptr(msghdr.msg_iov as _, msghdr.msg_iovlen as _));
        let info = socket.recvmsg(&mut iovecs, flags)?;

        // Handle source address return (if requested)
        if !msghdr.msg_name.is_null() {
            // The `msg_name` field points to a caller-allocated buffer that is used to return the source address if the socket is unconnected.
            // The caller should set msg_namelen to the size of this buffer before this call; upon return from a successful call, msg_namelen
            // will contain the size of the returned address.
            if let Some(address) = info.address {
                msghdr.msg_namelen =
                    write_sockaddr_with_max_len(address, msghdr.msg_name, msghdr.msg_namelen)?;
            }
        }

        // Initialize flags output field
        msghdr.msg_flags = 0;
        // Set `MSG_TRUNC` if received more data than buffer space
        if info.bytes_read != info.bytes_total {
            msghdr.msg_flags |= MessageFlags::MSG_TRUNC.bits();
        }

        // Ancillary data processing setup
        let cmsg_header_size = size_of::<ctypes::cmsghdr>();
        let ancillary_data_buffer_size = msghdr.msg_controllen as usize;
        let msg_control_ptr = msghdr.msg_control as *const u8;
        let mut ancillary_data_bytes_writen = 0;
        // Process each ancillary data item
        for cmsg_data in info.ancillary_data {
            let cmsg_data_size = cmsg_data.size();
            let expect_size = cmsg_header_size + cmsg_data_size;
            let minium_size = cmsg_data.minium_size();
            let space_available = ancillary_data_buffer_size - ancillary_data_bytes_writen;
            // cmsg_bytes = cmsg_header + cmsg_data
            let cmsg_bytes = if space_available < cmsg_header_size + minium_size {
                // Not enough space even for minimal data
                Vec::new()
            } else {
                let (cmsg_level, cmsg_type, mut cmsg_data_bytes) = cmsg_data.parse(flags)?;
                // If the space allocated for receiving incoming ancillary data is
                // too small then the ancillary data is truncated to the number of
                // headers that will fit in the supplied buffer (see unix.7)
                cmsg_data_bytes.truncate(space_available - cmsg_header_size);

                pack_cmsg(cmsg_level, cmsg_type, cmsg_data_bytes)
            };

            // Handle truncation flags
            let truncated = cmsg_bytes.len() < expect_size;
            if truncated {
                msghdr.msg_flags |= MessageFlags::MSG_CTRUNC.bits();
            }

            if cmsg_bytes.len() < cmsg_header_size {
                // Can't fit the header, so stop trying to write
                break;
            }

            // Copy to user-space buffer if valid
            if !cmsg_bytes.is_empty() {
                unsafe {
                    core::ptr::copy_nonoverlapping(
                        cmsg_bytes.as_ptr(),
                        (msg_control_ptr.add(ancillary_data_bytes_writen)) as *mut u8,
                        cmsg_bytes.len(),
                    )
                };
                ancillary_data_bytes_writen += cmsg_bytes.len();
                // Align for next CMsg
                if !truncated {
                    ancillary_data_bytes_writen = cmsg_align(ancillary_data_bytes_writen);
                }
            }
        }

        // Update user-space `msghdr` structure
        unsafe { *msg = msghdr };

        match flags.contains(MessageFlags::MSG_TRUNC) {
            true => Ok(info.bytes_total), // Return actual packet size
            false => Ok(info.bytes_read), // Return received bytes
        }
    })
}

/// Creates a pair of Unix domain sockets and stores the file descriptors in `sv`
///
/// This system call only works for UNIX domain sockets (AF_UNIX), which are used for communication
/// between processes on the same machine. It cannot be used for communication over the network (e.g.,
/// using AF_INET or AF_INET6). The created socket pair is anonymous, meaning it does not require
/// a pathname, and is typically used for communication between related processes (e.g., parent-child processes)
pub fn sys_socketpair(domain: c_int, socktype: c_int, protocol: c_int, sv: &mut [c_int]) -> c_int {
    syscall_body!(sys_socketpair, {
        let domain = SocketDomain::try_from(domain as u16)?;
        if domain != SocketDomain::Unix {
            return Err(LinuxError::EAFNOSUPPORT);
        }
        let mut flags = OpenFlags::empty();
        let socktype = socktype as u32;
        let nonblock = (socktype & ctypes::SOCK_NONBLOCK) != 0;
        if socktype & ctypes::SOCK_CLOEXEC != 0 {
            flags |= OpenFlags::O_CLOEXEC;
        }
        let socktype = socktype & !ctypes::SOCK_CLOEXEC & !ctypes::SOCK_NONBLOCK;
        let socktype = SocketType::try_from(socktype)?;
        info!("sys_socketpair <= domain: {domain:?}, socktype: {socktype:?}, protocol: {protocol}, sv pointer: {:#x}", sv.as_ptr() as usize);
        info!("nonblock: {nonblock}, cloexec: {flags:?}");
        let (sk1, sk2) = UnixSocket::create_socket_pair(socktype, nonblock);
        sv[0] = add_file_like(sk1, flags)?;
        sv[1] = add_file_like(sk2, flags)?;
        info!("create sv[0] {}, sv[1] {}", sv[0], sv[1]);
        Ok(0)
    })
}

fn socket_from_fd(fd: i32) -> LinuxResult<Arc<Socket>> {
    get_file_like(fd)?
        .into_any()
        .downcast::<Socket>()
        .map_err(|_| LinuxError::ENOTSOCK)
}

/*
* the following functions refers to macros in musl
* see https://www.man7.org/linux/man-pages/man3/cmsg.3.html
*/

/// ```c
/// #define CMSG_ALIGN(len) (((len) + sizeof (size_t) - 1) & (size_t) ~(sizeof (size_t) - 1))
/// ```
fn cmsg_align(len: usize) -> usize {
    const ALIGNMENT: usize = core::mem::size_of::<usize>();
    (len + ALIGNMENT - 1) & !(ALIGNMENT - 1)
}

fn pack_cmsg(cmsg_level: i32, cmsg_type: i32, cmsg_data_bytes: Vec<u8>) -> Vec<u8> {
    let cmsg_header_size = size_of::<ctypes::cmsghdr>();
    let cmsg_len = cmsg_header_size + cmsg_data_bytes.len();
    let cmsghdr = ctypes::cmsghdr {
        cmsg_len: cmsg_len as _,
        __pad1: 0,
        cmsg_level,
        cmsg_type,
    };
    let mut buffer = Vec::with_capacity(cmsg_len);
    unsafe {
        let hdr_ptr = &cmsghdr as *const _ as *const u8;
        buffer.extend_from_slice(core::slice::from_raw_parts(hdr_ptr, cmsg_header_size));
    }
    buffer.extend(cmsg_data_bytes);
    buffer
}
