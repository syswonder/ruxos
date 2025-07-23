/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
//! Unix socket implementation
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use alloc::sync::Weak;
use alloc::vec::Vec;
use axerrno::{LinuxError, LinuxResult};
use axio::PollState;
use spin::mutex::Mutex;

use core::sync::atomic::{AtomicBool, Ordering};
use iovec::{IoVecsInput, IoVecsOutput};

use crate::address::{resolve_unix_socket_addr, SocketAddress, UnixSocketAddr};
use crate::message::{ControlMessageData, MessageFlags, MessageQueue, MessageReadInfo};
use crate::socket::{Socket, SocketType};
use crate::socket_node::bind_socket_node;
use crate::ShutdownFlags;

const UNIX_DEFAULT_SIZE: usize = 65536;

/// Represents a UNIX domain socket implementation.
///
/// UNIX domain sockets provide inter-process communication on the same host system.
/// They support both stream-oriented (SOCK_STREAM) and datagram (SOCK_DGRAM) semantics.
pub struct UnixSocket {
    /// The type of socket (stream or datagram)
    socktype: SocketType,
    /// Whether the socket is in non-blocking mode
    nonblock: AtomicBool,
    /// The internal state protected by a mutex
    inner: Mutex<UnixSocketInner>,
}

/// Internal state of a UNIX domain socket
pub struct UnixSocketInner {
    /// Queue for incoming messages
    /// - Stream sockets (preserving message boundaries because of ancillary data)
    /// - Datagram sockets (maintaining packet boundaries)
    messages: MessageQueue,
    /// Local address the socket is bound to (filesystem path for named sockets)
    local_address: Option<SocketAddress>,
    /// Address of the connected peer (if connected)
    peer_address: Option<SocketAddress>,
    /// Current connection state
    state: UnixSocketState,
    /// Read shutdown flag (SHUT_RD):
    /// - When true: further receives are disallowed
    /// - Note: Any remaining data in message queue can still be read
    /// - After shutdown: recv() returns EOF (0 bytes) once queue is empty
    shutdown_read: bool,
    /// Write shutdown flag (SHUT_WR):
    /// - When true: further sends are disallowed
    /// - Triggers peer_hangup on remote socket
    /// - Does not affect already-queued messages
    shutdown_write: bool,
    /// Peer disconnect notification:
    /// - Set when remote end shuts down or closes
    /// - Pollers will receive POLLHUP event
    /// - Further writes will typically fail with `EPIPE`
    peer_hangup: bool,
}

impl UnixSocketInner {
    /// Gets the connected peer socket if available
    pub fn peer(&self) -> Option<Arc<Socket>> {
        match &self.state {
            UnixSocketState::Connected(peer) => peer.upgrade(),
            _ => None,
        }
    }
}

/// Represents the connection state of a UNIX socket
pub enum UnixSocketState {
    /// Disconnected
    Disconnected,
    /// Listening for incoming connections (stream sockets only)
    Listening(AcceptQueue),
    /// Connected to a peer socket (using Weak to avoid reference cycles)
    Connected(Weak<Socket>),
    /// Socket has been closed
    Closed,
}

/// Queue for pending incoming connections (stream sockets)
pub struct AcceptQueue {
    /// Queue of pending connections
    sockets: VecDeque<Arc<Socket>>,
    /// Maximum number of pending connections allowed
    backlog: usize,
}

impl AcceptQueue {
    /// Default backlog size when not specified
    const DEFAULT_BACKLOG: usize = 1024;

    /// Creates a new accept queue with specified backlog
    fn new(backlog: usize) -> Self {
        AcceptQueue {
            sockets: VecDeque::with_capacity(backlog),
            backlog,
        }
    }

    fn set_backlog(&mut self, backlog: usize) {
        self.backlog = backlog;
    }
}

impl UnixSocket {
    /// Creates a new UNIX domain socket
    pub fn create_socket(socktype: SocketType, nonblock: bool) -> Arc<Socket> {
        Arc::new(Socket::Unix(UnixSocket {
            socktype,
            nonblock: AtomicBool::new(nonblock),
            inner: Mutex::new(UnixSocketInner {
                messages: MessageQueue::new(UNIX_DEFAULT_SIZE),
                local_address: None,
                peer_address: None,
                state: UnixSocketState::Disconnected,
                shutdown_read: false,
                shutdown_write: false,
                peer_hangup: false,
            }),
        }))
    }

    /// Creates a pair of connected UNIX sockets (like pipe())
    pub fn create_socket_pair(socktype: SocketType, nonblock: bool) -> (Arc<Socket>, Arc<Socket>) {
        let left = Self::create_socket(socktype, nonblock);
        let right = Self::create_socket(socktype, nonblock);
        {
            let mut left_inner = left.as_unix_socket().inner.lock();
            left_inner.state = UnixSocketState::Connected(Arc::downgrade(&right));
            left_inner.local_address = Some(SocketAddress::Unix(UnixSocketAddr::Unamed));
        }
        {
            let mut right_inner = right.as_unix_socket().inner.lock();
            right_inner.state = UnixSocketState::Connected(Arc::downgrade(&left));
            right_inner.local_address = Some(SocketAddress::Unix(UnixSocketAddr::Unamed));
        }
        (left, right)
    }

    /// Returns the socket type (stream or datagram)
    pub fn socket_type(&self) -> SocketType {
        self.socktype
    }

    /// Checks if the socket is in non-blocking mode
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Relaxed)
    }

    /// Sets the socket's non-blocking mode
    pub fn set_nonblocking(&self, nonblock: bool) {
        self.nonblock.store(nonblock, Ordering::Relaxed);
    }

    /// Checks if the socket is in listening state
    pub fn is_listening(&self) -> bool {
        matches!(self.inner.lock().state, UnixSocketState::Listening(_))
    }

    /// Binds the socket to a filesystem path
    pub fn bind(&self, self_socket: Arc<Socket>, address: SocketAddress) -> LinuxResult {
        if let SocketAddress::Unix(ref unix_addr) = address {
            let mut inner = self.inner.lock();
            // Check if the socket is already bound to an address.
            if inner.local_address.is_some() {
                return Err(LinuxError::EINVAL);
            }
            match unix_addr {
                UnixSocketAddr::PathName(ref path) => bind_socket_node(self_socket, path)?,
                UnixSocketAddr::Unamed => {
                    unreachable!("won't parse a unix address to Unamed")
                }
                UnixSocketAddr::Abstract(_) => todo!(),
            }
            inner.local_address = Some(address);
            return Ok(());
        }
        Err(LinuxError::EINVAL)
    }

    /// Starts listening for incoming connections (stream sockets only)
    pub fn listen(&self, backlog: i32) -> LinuxResult {
        if self.socktype == SocketType::Datagram {
            return Err(LinuxError::EOPNOTSUPP);
        }
        let mut inner = self.inner.lock();
        let backlog = if backlog < 0 {
            AcceptQueue::DEFAULT_BACKLOG
        } else {
            backlog as usize
        };
        let is_bound = inner.local_address.is_some();
        match inner.state {
            UnixSocketState::Disconnected if is_bound => {
                inner.state = UnixSocketState::Listening(AcceptQueue::new(backlog));
                Ok(())
            }
            UnixSocketState::Listening(ref mut accept_queue) => {
                accept_queue.set_backlog(backlog);
                Ok(())
            }
            _ => Err(LinuxError::EINVAL),
        }
    }

    /// Accepts an incoming connection (stream sockets only)
    pub fn accept(&self) -> LinuxResult<Arc<Socket>> {
        if self.socktype == SocketType::Datagram {
            return Err(LinuxError::EOPNOTSUPP);
        }
        self.block_on(
            || {
                let mut inner = self.inner.lock();
                match &mut inner.state {
                    UnixSocketState::Listening(accept_queue) => {
                        accept_queue.sockets.pop_front().ok_or(LinuxError::EAGAIN)
                    }
                    _ => Err(LinuxError::EINVAL),
                }
            },
            MessageFlags::empty(),
        )
    }

    /// Connects to another UNIX socket
    pub fn connect(&self, self_socket: Arc<Socket>, address: SocketAddress) -> LinuxResult {
        let peer_socket = resolve_unix_socket_addr(&address)?;
        let peer = peer_socket.as_unix_socket();
        if self.socktype != peer.socktype {
            return Err(LinuxError::EPROTOTYPE);
        }
        let mut self_inner = self.inner.lock();
        self_inner.peer_address = Some(address);
        match self.socktype {
            SocketType::Datagram => {
                self_inner.state = UnixSocketState::Connected(Arc::downgrade(&peer_socket))
            }
            SocketType::Stream => {
                match self_inner.state {
                    UnixSocketState::Disconnected => {}
                    UnixSocketState::Connected(_) => return Err(LinuxError::EISCONN),
                    _ => return Err(LinuxError::EINVAL),
                }
                let mut listener = peer.inner.lock();
                let listener_capacity = listener.messages.capacity();
                let listener_address = listener.local_address.clone();
                match listener.state {
                    UnixSocketState::Listening(ref mut accept_queue) => {
                        if accept_queue.sockets.len() >= accept_queue.backlog {
                            return Err(LinuxError::EAGAIN);
                        }
                        let new_unix_socket = UnixSocket {
                            socktype: SocketType::Stream,
                            nonblock: AtomicBool::new(false),
                            inner: Mutex::new(UnixSocketInner {
                                messages: MessageQueue::new(listener_capacity),
                                local_address: listener_address,
                                peer_address: Some(
                                    self_inner
                                        .local_address
                                        .clone()
                                        .unwrap_or(SocketAddress::Unix(UnixSocketAddr::default())),
                                ),
                                state: UnixSocketState::Connected(Arc::downgrade(&self_socket)),
                                shutdown_read: false,
                                shutdown_write: false,
                                peer_hangup: false,
                            }),
                        };
                        let new_socket = Arc::new(Socket::Unix(new_unix_socket));
                        self_inner.state = UnixSocketState::Connected(Arc::downgrade(&new_socket));
                        accept_queue.sockets.push_back(new_socket);
                    }
                    _ => return Err(LinuxError::ECONNREFUSED),
                }
            }
        }
        Ok(())
    }

    /// If getsockname() is called on an unbound UNIX domain socket,
    /// the system will return success, but the `sun_path` in the
    /// address structure will be empty (indicating an unbound state).
    pub fn local_addr(&self) -> LinuxResult<SocketAddress> {
        let inner = self.inner.lock();
        match &inner.local_address {
            Some(address) => Ok(address.clone()),
            None => Ok(SocketAddress::Unix(UnixSocketAddr::default())),
        }
    }

    /// Gets the peer address if connected
    pub fn peer_addr(&self) -> LinuxResult<SocketAddress> {
        self.inner
            .lock()
            .peer_address
            .clone()
            .ok_or(LinuxError::ENOTCONN)
    }

    /// Find peer unix socket and write message to it's `MessageQueue`.
    pub fn sendmsg(
        &self,
        src_data: &IoVecsInput,
        dst_address: Option<SocketAddress>,
        ancillary_data: &mut Vec<ControlMessageData>,
        flags: MessageFlags,
    ) -> LinuxResult<usize> {
        self.block_on(
            || {
                let (local_address, connected_peer) = {
                    let inner = self.inner.lock();
                    if inner.shutdown_write {
                        // The local end has been shut down on a connection oriented socket.
                        return Err(LinuxError::EPIPE);
                    }
                    (inner.local_address.clone(), inner.peer())
                };
                let peer = match (connected_peer, dst_address.as_ref(), self.socktype) {
                    (None, None, _) => {
                        // The socket is not connected, and no target has been given.
                        return Err(LinuxError::ENOTCONN);
                    }
                    (None, Some(_), SocketType::Stream) => {
                        return Err(LinuxError::ENOTCONN);
                    }
                    (None, Some(address), SocketType::Datagram) => {
                        // The unix Dgram socket is not connected, but a target address has been given.
                        resolve_unix_socket_addr(address)?
                    }
                    (Some(peer), None, _) => peer,
                    (Some(_), Some(_), _) => {
                        //The connection-mode socket was connected already but a recipient was specified.
                        return Err(LinuxError::EISCONN);
                    }
                };
                let mut peer_inner = peer.as_unix_socket().inner.lock();
                if self.socktype == SocketType::Stream {
                    peer_inner
                        .messages
                        .write_stream(src_data, local_address, ancillary_data)
                } else {
                    peer_inner
                        .messages
                        .write_dgram(src_data, local_address, ancillary_data)
                }
            },
            flags,
        )
    }

    /// Receives a message from the socket
    pub fn recvmsg(
        &self,
        dst_data: &mut IoVecsOutput,
        flags: MessageFlags,
    ) -> LinuxResult<MessageReadInfo> {
        self.block_on(
            || {
                let mut inner = self.inner.lock();
                let info = match self.socktype {
                    SocketType::Stream => {
                        if dst_data.avaliable() == 0 {
                            Ok(MessageReadInfo::default())
                        } else if flags.contains(MessageFlags::MSG_PEEK) {
                            inner.messages.peek_stream(dst_data)
                        } else {
                            inner.messages.read_stream(dst_data)
                        }
                    }
                    SocketType::Datagram => {
                        if flags.contains(MessageFlags::MSG_PEEK) {
                            inner.messages.peek_dgram(dst_data)
                        } else {
                            inner.messages.read_dgram(dst_data)
                        }
                    }
                }?;
                // Unix domain sockets can send empty messages, so we need to check if the read bytes are zero with address
                if info.bytes_read == 0 && !inner.shutdown_read && info.address.is_none() {
                    return Err(LinuxError::EAGAIN);
                }
                Ok(info)
            },
            flags,
        )
    }

    /// Checks the socket's I/O readiness state
    pub fn poll(&self) -> LinuxResult<PollState> {
        let inner = self.inner.lock();
        match self.socktype {
            SocketType::Stream => match inner.state {
                UnixSocketState::Disconnected => Ok(PollState::default()),
                UnixSocketState::Listening(ref accept_queue) => {
                    let readable = accept_queue.sockets.is_empty();
                    Ok(PollState {
                        readable,
                        writable: false,
                        pollhup: false,
                    })
                }
                UnixSocketState::Connected(_) => {
                    let readable = !inner.messages.is_empty();
                    let writable = inner.messages.available_capacity() > 0 && !inner.shutdown_write;
                    Ok(PollState {
                        readable,
                        writable,
                        pollhup: inner.peer_hangup,
                    })
                }
                UnixSocketState::Closed => {
                    let readable = !inner.messages.is_empty();
                    Ok(PollState {
                        readable,
                        writable: false,
                        pollhup: inner.peer_hangup,
                    })
                }
            },
            SocketType::Datagram => {
                let readable = !inner.messages.is_empty();
                let writable = inner.messages.available_capacity() > 0 && !inner.shutdown_write;
                Ok(PollState {
                    readable,
                    writable,
                    pollhup: inner.peer_hangup,
                })
            }
        }
    }

    /// Shuts down part or all of the socket connection
    pub fn shutdown(&self, how: ShutdownFlags) -> LinuxResult {
        let mut inner = self.inner.lock();
        let peer = inner.peer().ok_or(LinuxError::ENOTCONN)?;
        let mut peer_inner = peer.as_unix_socket().inner.lock();
        if how.contains(ShutdownFlags::WRITE) {
            inner.shutdown_write = true;
            peer_inner.peer_hangup = true;
        }
        if how.contains(ShutdownFlags::READ) {
            inner.shutdown_read = true;
        }
        Ok(())
    }

    /// Helper for blocking/non-blocking operations
    fn block_on<F, T>(&self, mut f: F, flags: MessageFlags) -> LinuxResult<T>
    where
        F: FnMut() -> LinuxResult<T>,
    {
        if flags.contains(MessageFlags::MSG_DONTWAIT) || self.is_nonblocking() {
            return f();
        }
        loop {
            let res = f();
            match res {
                Ok(t) => return Ok(t),
                Err(LinuxError::EAGAIN) => ruxtask::yield_now(),
                Err(e) => return Err(e),
            }
        }
    }
}

impl Drop for UnixSocket {
    fn drop(&mut self) {
        if let UnixSocketState::Connected(ref peer) = self.inner.lock().state {
            if let Some(peer_socket) = peer.upgrade() {
                let mut peer_inner = peer_socket.as_unix_socket().inner.lock();
                peer_inner.state = UnixSocketState::Closed;
                peer_inner.peer_hangup = true;
            }
        }
    }
}
