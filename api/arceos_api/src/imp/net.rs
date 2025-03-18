/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

use crate::io::AxPollState;
use axerrno::AxResult;
use core::net::{IpAddr, SocketAddr};
use ruxnet::{TcpSocket, UdpSocket};

/// A handle to a TCP socket.
pub struct AxTcpSocketHandle(TcpSocket);

/// A handle to a UDP socket.
pub struct AxUdpSocketHandle(UdpSocket);

////////////////////////////////////////////////////////////////////////////////
// TCP socket
////////////////////////////////////////////////////////////////////////////////

pub fn ax_tcp_socket() -> AxTcpSocketHandle {
    AxTcpSocketHandle(TcpSocket::new(false))
}

pub fn ax_tcp_socket_addr(socket: &AxTcpSocketHandle) -> AxResult<SocketAddr> {
    socket.0.local_addr()
}

pub fn ax_tcp_peer_addr(socket: &AxTcpSocketHandle) -> AxResult<SocketAddr> {
    socket.0.peer_addr()
}

pub fn ax_tcp_set_nonblocking(socket: &AxTcpSocketHandle, nonblocking: bool) -> AxResult {
    socket.0.set_nonblocking(nonblocking);
    Ok(())
}

pub fn ax_tcp_connect(socket: &AxTcpSocketHandle, addr: SocketAddr) -> AxResult {
    socket.0.connect(addr)
}

pub fn ax_tcp_bind(socket: &AxTcpSocketHandle, addr: SocketAddr) -> AxResult {
    socket.0.bind(addr)
}

pub fn ax_tcp_listen(socket: &mut AxTcpSocketHandle, _backlog: usize) -> AxResult {
    socket.0.listen()
}

pub fn ax_tcp_accept(socket: &AxTcpSocketHandle) -> AxResult<(AxTcpSocketHandle, SocketAddr)> {
    let new_sock = socket.0.accept()?;
    let addr = new_sock.peer_addr()?;
    Ok((AxTcpSocketHandle(new_sock), addr))
}

pub fn ax_tcp_send(socket: &AxTcpSocketHandle, buf: &[u8]) -> AxResult<usize> {
    socket.0.send(buf)
}

pub fn ax_tcp_recv(socket: &AxTcpSocketHandle, buf: &mut [u8]) -> AxResult<usize> {
    socket.0.recv(buf, 0)
}

pub fn ax_tcp_poll(socket: &AxTcpSocketHandle) -> AxResult<AxPollState> {
    socket.0.poll()
}

pub fn ax_tcp_shutdown(socket: &AxTcpSocketHandle) -> AxResult {
    socket.0.shutdown()
}

////////////////////////////////////////////////////////////////////////////////
// UDP socket
////////////////////////////////////////////////////////////////////////////////

pub fn ax_udp_socket() -> AxUdpSocketHandle {
    AxUdpSocketHandle(UdpSocket::new())
}

pub fn ax_udp_socket_addr(socket: &AxUdpSocketHandle) -> AxResult<SocketAddr> {
    socket.0.local_addr()
}

pub fn ax_udp_peer_addr(socket: &AxUdpSocketHandle) -> AxResult<SocketAddr> {
    socket.0.peer_addr()
}

pub fn ax_udp_set_nonblocking(socket: &AxUdpSocketHandle, nonblocking: bool) -> AxResult {
    socket.0.set_nonblocking(nonblocking);
    Ok(())
}

pub fn ax_udp_bind(socket: &AxUdpSocketHandle, addr: SocketAddr) -> AxResult {
    socket.0.bind(addr)
}

pub fn ax_udp_recv_from(
    socket: &AxUdpSocketHandle,
    buf: &mut [u8],
) -> AxResult<(usize, SocketAddr)> {
    socket.0.recv_from(buf)
}

pub fn ax_udp_peek_from(
    socket: &AxUdpSocketHandle,
    buf: &mut [u8],
) -> AxResult<(usize, SocketAddr)> {
    socket.0.peek_from(buf)
}

pub fn ax_udp_send_to(socket: &AxUdpSocketHandle, buf: &[u8], addr: SocketAddr) -> AxResult<usize> {
    socket.0.send_to(buf, addr)
}

pub fn ax_udp_connect(socket: &AxUdpSocketHandle, addr: SocketAddr) -> AxResult {
    socket.0.connect(addr)
}

pub fn ax_udp_send(socket: &AxUdpSocketHandle, buf: &[u8]) -> AxResult<usize> {
    socket.0.send(buf)
}

pub fn ax_udp_recv(socket: &AxUdpSocketHandle, buf: &mut [u8]) -> AxResult<usize> {
    socket.0.recv(buf)
}

pub fn ax_udp_poll(socket: &AxUdpSocketHandle) -> AxResult<AxPollState> {
    socket.0.poll()
}

////////////////////////////////////////////////////////////////////////////////
// Miscellaneous
////////////////////////////////////////////////////////////////////////////////

pub fn ax_dns_query(domain_name: &str) -> AxResult<alloc::vec::Vec<IpAddr>> {
    ruxnet::dns_query(domain_name)
}

pub fn ax_poll_interfaces() -> AxResult {
    ruxnet::poll_interfaces();
    Ok(())
}
