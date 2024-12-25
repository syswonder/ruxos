/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/

use alloc::{sync::Arc, vec};
use axerrno::{ax_err, AxError, AxResult, LinuxError, LinuxResult};
use axio::PollState;
use axsync::Mutex;
use core::ffi::c_char;
use core::net::SocketAddr;
use core::sync::atomic::{AtomicBool, Ordering};
use spin::RwLock;

use lazy_init::LazyInit;

use smoltcp::socket::tcp::SocketBuffer;

use hashbrown::HashMap;

use ruxfs::root::{create_file, lookup};
use ruxtask::yield_now;

const SOCK_ADDR_UN_PATH_LEN: usize = 108;

/// rust form for ctype sockaddr_un
#[derive(Clone, Copy, Debug)]
pub struct SocketAddrUnix {
    /// AF_UNIX
    pub sun_family: u16,
    /// socket path
    pub sun_path: [c_char; SOCK_ADDR_UN_PATH_LEN], /* Pathname */
}

impl SocketAddrUnix {
    /// Sets the socket address to the specified new address.
    pub fn set_addr(&mut self, new_addr: &SocketAddrUnix) {
        self.sun_family = new_addr.sun_family;
        self.sun_path = new_addr.sun_path;
    }
}

//To avoid owner question of FDTABLE outside and UnixTable in this crate we split the unixsocket
struct UnixSocketInner<'a> {
    pub addr: Mutex<SocketAddrUnix>,
    pub buf: SocketBuffer<'a>,
    pub peer_socket: Option<usize>,
    pub status: UnixSocketStatus,
}

impl<'a> UnixSocketInner<'a> {
    pub fn new() -> Self {
        Self {
            addr: Mutex::new(SocketAddrUnix {
                sun_family: 1, //AF_UNIX
                sun_path: [0; SOCK_ADDR_UN_PATH_LEN],
            }),
            buf: SocketBuffer::new(vec![0; 64 * 1024]),
            peer_socket: None,
            status: UnixSocketStatus::Closed,
        }
    }

    pub fn get_addr(&self) -> SocketAddrUnix {
        self.addr.lock().clone()
    }

    pub fn get_peersocket(&self) -> Option<usize> {
        self.peer_socket
    }

    pub fn set_peersocket(&mut self, peer: usize) {
        self.peer_socket = Some(peer)
    }

    pub fn get_state(&self) -> UnixSocketStatus {
        self.status
    }

    pub fn set_state(&mut self, state: UnixSocketStatus) {
        self.status = state
    }

    pub fn can_accept(&mut self) -> bool {
        match self.status {
            UnixSocketStatus::Listening => !self.buf.is_empty(),
            _ => false,
        }
    }

    pub fn may_recv(&mut self) -> bool {
        match self.status {
            UnixSocketStatus::Connected => true,
            //State::FinWait1 | State::FinWait2 => true,
            _ if !self.buf.is_empty() => true,
            _ => false,
        }
    }

    pub fn can_recv(&mut self) -> bool {
        if !self.may_recv() {
            return false;
        }

        !self.buf.is_empty()
    }

    pub fn may_send(&mut self) -> bool {
        match self.status {
            UnixSocketStatus::Connected => true,
            //State::CloseWait => true,
            _ => false,
        }
    }

    pub fn can_send(&mut self) -> bool {
        self.may_send()
    }
}

/// unix domain socket.
pub struct UnixSocket {
    sockethandle: Option<usize>,
    unixsocket_type: UnixSocketType,
    nonblock: AtomicBool,
}

// now there is no real inode, this func is to check whether file exists
// TODO: if inode impl, this should return inode
fn get_inode(addr: SocketAddrUnix) -> AxResult<usize> {
    let slice = unsafe { core::slice::from_raw_parts(addr.sun_path.as_ptr(), addr.sun_path.len()) };

    let socket_path = unsafe {
        core::ffi::CStr::from_ptr(slice.as_ptr())
            .to_str()
            .expect("Invalid UTF-8 string")
    };
    let _vfsnode = match lookup(None, socket_path) {
        Ok(node) => node,
        Err(_) => {
            return Err(AxError::NotFound);
        }
    };

    Err(AxError::Unsupported)
}

fn create_socket_file(addr: SocketAddrUnix) -> AxResult<usize> {
    let slice = unsafe { core::slice::from_raw_parts(addr.sun_path.as_ptr(), addr.sun_path.len()) };

    let socket_path = unsafe {
        core::ffi::CStr::from_ptr(slice.as_ptr())
            .to_str()
            .expect("Invalid UTF-8 string")
    };
    let _vfsnode = create_file(None, socket_path)?;
    Err(AxError::Unsupported)
}

struct HashMapWarpper<'a> {
    inner: HashMap<usize, Arc<Mutex<UnixSocketInner<'a>>>>,
    index_allcator: Mutex<usize>,
}
impl<'a> HashMapWarpper<'a> {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            index_allcator: Mutex::new(0),
        }
    }
    pub fn find<F>(&self, predicate: F) -> Option<(&usize, &Arc<Mutex<UnixSocketInner<'a>>>)>
    where
        F: Fn(&Arc<Mutex<UnixSocketInner<'_>>>) -> bool,
    {
        self.inner.iter().find(|(_k, v)| predicate(v))
    }

    pub fn add(&mut self, value: Arc<Mutex<UnixSocketInner<'a>>>) -> Option<usize> {
        let index_allcator = self.index_allcator.get_mut();
        while self.inner.contains_key(index_allcator) {
            *index_allcator += 1;
        }
        self.inner.insert(*index_allcator, value);
        Some(*index_allcator)
    }

    pub fn replace_handle(&mut self, old: usize, new: usize) -> Option<usize> {
        if let Some(value) = self.inner.remove(&old) {
            self.inner.insert(new, value);
        }
        Some(new)
    }

    pub fn get(&self, id: usize) -> Option<&Arc<Mutex<UnixSocketInner<'a>>>> {
        self.inner.get(&id)
    }

    pub fn get_mut(&mut self, id: usize) -> Option<&mut Arc<Mutex<UnixSocketInner<'a>>>> {
        self.inner.get_mut(&id)
    }

    pub fn remove(&mut self, id: usize) -> Option<Arc<Mutex<UnixSocketInner<'a>>>> {
        self.inner.remove(&id)
    }
}
static UNIX_TABLE: LazyInit<RwLock<HashMapWarpper>> = LazyInit::new();

/// unix socket type
#[derive(Debug, Clone, Copy)]
pub enum UnixSocketType {
    /// A stream-oriented Unix domain socket.
    SockStream,
    /// A datagram-oriented Unix domain socket.
    SockDgram,
    /// A sequenced packet Unix domain socket.
    SockSeqpacket,
}

// State transitions:
// CLOSED -(connect)-> BUSY -> CONNECTING -> CONNECTED -(shutdown)-> BUSY -> CLOSED
//       |
//       |-(listen)-> BUSY -> LISTENING -(shutdown)-> BUSY -> CLOSED
//       |
//        -(bind)-> BUSY -> CLOSED
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UnixSocketStatus {
    Closed,
    Busy,
    Connecting,
    Connected,
    Listening,
}

impl UnixSocket {
    /// create a new socket
    /// only support sock_stream
    pub fn new(_type: UnixSocketType) -> Self {
        match _type {
            UnixSocketType::SockDgram | UnixSocketType::SockSeqpacket => unimplemented!(),
            UnixSocketType::SockStream => {
                let mut unixsocket = UnixSocket {
                    sockethandle: None,
                    unixsocket_type: _type,
                    nonblock: AtomicBool::new(false),
                };
                let handle = UNIX_TABLE
                    .write()
                    .add(Arc::new(Mutex::new(UnixSocketInner::new())))
                    .unwrap();
                unixsocket.set_sockethandle(handle);
                unixsocket
            }
        }
    }

    /// Sets the socket handle.
    pub fn set_sockethandle(&mut self, fd: usize) {
        self.sockethandle = Some(fd);
    }

    /// Returns the socket handle.
    pub fn get_sockethandle(&self) -> usize {
        self.sockethandle.unwrap()
    }

    /// Returns the peer socket handle, if available.
    pub fn get_peerhandle(&self) -> Option<usize> {
        UNIX_TABLE
            .read()
            .get(self.get_sockethandle())
            .unwrap()
            .lock()
            .get_peersocket()
    }

    /// Returns the current state of the socket.
    pub fn get_state(&self) -> UnixSocketStatus {
        UNIX_TABLE
            .read()
            .get(self.get_sockethandle())
            .unwrap()
            .lock()
            .status
    }

    /// Enqueues data into the socket buffer.
    /// returns the number of bytes enqueued, or an error if the socket is closed.
    pub fn enqueue_buf(&mut self, data: &[u8]) -> AxResult<usize> {
        match self.get_state() {
            UnixSocketStatus::Closed => Err(AxError::BadState),
            _ => Ok(UNIX_TABLE
                .write()
                .get_mut(self.get_sockethandle())
                .unwrap()
                .lock()
                .buf
                .enqueue_slice(data)),
        }
    }

    /// Dequeues data from the socket buffer.
    /// return the number of bytes dequeued, or a BadState error if the socket is closed or a WouldBlock error if buffer is empty.
    pub fn dequeue_buf(&mut self, data: &mut [u8]) -> AxResult<usize> {
        match self.get_state() {
            UnixSocketStatus::Closed => Err(AxError::BadState),
            _ => {
                if UNIX_TABLE
                    .write()
                    .get_mut(self.get_sockethandle())
                    .unwrap()
                    .lock()
                    .buf
                    .is_empty()
                {
                    return Err(AxError::WouldBlock);
                }
                Ok(UNIX_TABLE
                    .write()
                    .get_mut(self.get_sockethandle())
                    .unwrap()
                    .lock()
                    .buf
                    .dequeue_slice(data))
            }
        }
    }

    /// Binds the socket to a specified address, get inode number of the address as handle
    // TODO: bind to file system
    pub fn bind(&mut self, addr: SocketAddrUnix) -> LinuxResult {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Closed => {
                {
                    match get_inode(addr) {
                        Ok(inode_addr) => {
                            UNIX_TABLE
                                .write()
                                .replace_handle(self.get_sockethandle(), inode_addr);
                            self.set_sockethandle(inode_addr);
                        }
                        Err(AxError::NotFound) => match create_socket_file(addr) {
                            Ok(inode_addr) => {
                                UNIX_TABLE
                                    .write()
                                    .replace_handle(self.get_sockethandle(), inode_addr);
                                self.set_sockethandle(inode_addr);
                            }
                            _ => {
                                warn!("unix socket can not get real inode");
                            }
                        },
                        _ => {
                            warn!("unix socket can not get real inode");
                        }
                    }
                }
                let mut binding = UNIX_TABLE.write();
                let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
                socket_inner.addr.lock().set_addr(&addr);
                socket_inner.set_state(UnixSocketStatus::Busy);
                Ok(())
            }
            _ => Err(LinuxError::EINVAL),
        }
    }

    /// Sends data through the socket to the connected peer, push data into buffer of peer socket
    /// this will block if not connected by default
    pub fn send(&self, buf: &[u8]) -> LinuxResult<usize> {
        match self.unixsocket_type {
            UnixSocketType::SockDgram | UnixSocketType::SockSeqpacket => Err(LinuxError::ENOTCONN),
            UnixSocketType::SockStream => loop {
                let now_state = self.get_state();
                match now_state {
                    UnixSocketStatus::Connecting => {
                        if self.is_nonblocking() {
                            return Err(LinuxError::EINPROGRESS);
                        } else {
                            yield_now();
                        }
                    }
                    UnixSocketStatus::Connected => {
                        let peer_handle = UNIX_TABLE
                            .read()
                            .get(self.get_sockethandle())
                            .unwrap()
                            .lock()
                            .get_peersocket()
                            .unwrap();
                        return Ok(UNIX_TABLE
                            .write()
                            .get_mut(peer_handle)
                            .unwrap()
                            .lock()
                            .buf
                            .enqueue_slice(buf));
                    }
                    _ => {
                        return Err(LinuxError::ENOTCONN);
                    }
                }
            },
        }
    }

    /// Receives data from the socket, check if there any data in buffer
    /// this will block if not connected or buffer is empty by default
    pub fn recv(&self, buf: &mut [u8], _flags: i32) -> LinuxResult<usize> {
        match self.unixsocket_type {
            UnixSocketType::SockDgram | UnixSocketType::SockSeqpacket => Err(LinuxError::ENOTCONN),
            UnixSocketType::SockStream => loop {
                let now_state = self.get_state();
                match now_state {
                    UnixSocketStatus::Connecting => {
                        if self.is_nonblocking() {
                            return Err(LinuxError::EAGAIN);
                        } else {
                            yield_now();
                        }
                    }
                    UnixSocketStatus::Connected => {
                        if UNIX_TABLE
                            .read()
                            .get(self.get_sockethandle())
                            .unwrap()
                            .lock()
                            .buf
                            .is_empty()
                        {
                            if self.is_nonblocking() {
                                return Err(LinuxError::EAGAIN);
                            } else {
                                yield_now();
                            }
                        } else {
                            return Ok(UNIX_TABLE
                                .read()
                                .get(self.get_sockethandle())
                                .unwrap()
                                .lock()
                                .buf
                                .dequeue_slice(buf));
                        }
                    }
                    _ => {
                        return Err(LinuxError::ENOTCONN);
                    }
                }
            },
        }
    }

    /// Polls the socket's readiness for connection.
    fn poll_connect(&self) -> LinuxResult<PollState> {
        let writable = {
            let mut binding = UNIX_TABLE.write();
            let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
            if !socket_inner.get_peersocket().is_none() {
                socket_inner.set_state(UnixSocketStatus::Connected);
                true
            } else {
                false
            }
        };
        Ok(PollState {
            readable: false,
            writable,
            pollhup: false,
        })
    }

    /// Polls the socket's readiness for reading or writing.
    pub fn poll(&self) -> LinuxResult<PollState> {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Connecting => self.poll_connect(),
            UnixSocketStatus::Connected => {
                let remote_is_close = {
                    let remote_handle = self.get_peerhandle();
                    match remote_handle {
                        Some(handle) => {
                            let mut binding = UNIX_TABLE.write();
                            let remote_status = binding.get_mut(handle).unwrap().lock().get_state();
                            remote_status == UnixSocketStatus::Closed
                        }
                        None => {
                            return Err(LinuxError::ENOTCONN);
                        }
                    }
                };
                let mut binding = UNIX_TABLE.write();
                let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
                Ok(PollState {
                    readable: !socket_inner.may_recv() || socket_inner.can_recv(),
                    writable: !socket_inner.may_send() || socket_inner.can_send(),
                    pollhup: remote_is_close,
                })
            }
            UnixSocketStatus::Listening => {
                let mut binding = UNIX_TABLE.write();
                let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
                Ok(PollState {
                    readable: socket_inner.can_accept(),
                    writable: false,
                    pollhup: false,
                })
            }
            _ => Ok(PollState {
                readable: false,
                writable: false,
                pollhup: false,
            }),
        }
    }

    /// Returns the local address of the socket.
    pub fn local_addr(&self) -> LinuxResult<SocketAddr> {
        unimplemented!()
    }

    /// Returns the peer address of the socket.
    pub fn peer_addr(&self) -> AxResult<SocketAddrUnix> {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Connected | UnixSocketStatus::Listening => {
                let peer_sockethandle = self.get_peerhandle().unwrap();
                Ok(UNIX_TABLE
                    .read()
                    .get(peer_sockethandle)
                    .unwrap()
                    .lock()
                    .get_addr())
            }
            _ => Err(AxError::NotConnected),
        }
    }

    /// Connects the socket to a specified address, push info into remote socket
    pub fn connect(&mut self, addr: SocketAddrUnix) -> LinuxResult {
        let now_state = self.get_state();
        if now_state != UnixSocketStatus::Connecting && now_state != UnixSocketStatus::Connected {
            //a new block is needed to free rwlock
            {
                match get_inode(addr) {
                    Ok(inode_addr) => {
                        let binding = UNIX_TABLE.write();
                        let remote_socket = binding.get(inode_addr).unwrap();
                        if remote_socket.lock().get_state() != UnixSocketStatus::Listening {
                            error!("unix conncet error: remote socket not listening");
                            return Err(LinuxError::EFAULT);
                        }
                        let data = &self.get_sockethandle().to_ne_bytes();
                        let _res = remote_socket.lock().buf.enqueue_slice(data);
                    }
                    Err(AxError::NotFound) => return Err(LinuxError::ENOENT),
                    _ => {
                        warn!("unix socket can not get real inode");
                        let binding = UNIX_TABLE.write();
                        let (_remote_sockethandle, remote_socket) = binding
                            .find(|socket| socket.lock().addr.lock().sun_path == addr.sun_path)
                            .unwrap();
                        let data = &self.get_sockethandle().to_ne_bytes();
                        let _res = remote_socket.lock().buf.enqueue_slice(data);
                    }
                }
            }
            {
                let mut binding = UNIX_TABLE.write();
                let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
                socket_inner.set_state(UnixSocketStatus::Connecting);
            }
        }

        loop {
            let PollState { writable, .. } = self.poll_connect()?;
            if !writable {
                // When set to non_blocking, directly return inporgress
                if self.is_nonblocking() {
                    return Err(LinuxError::EINPROGRESS);
                } else {
                    yield_now();
                }
            } else if self.get_state() == UnixSocketStatus::Connected {
                return Ok(());
            } else {
                // When set to non_blocking, directly return inporgress
                if self.is_nonblocking() {
                    return Err(LinuxError::EINPROGRESS);
                }
                warn!("socket connect() failed")
            }
        }
    }

    /// Sends data to a specified address.
    pub fn sendto(&self, _buf: &[u8], _addr: SocketAddrUnix) -> LinuxResult<usize> {
        unimplemented!()
    }

    /// Receives data from the socket and returns the sender's address.
    pub fn recvfrom(&self, _buf: &mut [u8]) -> LinuxResult<(usize, Option<SocketAddr>)> {
        unimplemented!()
    }

    /// Listens for incoming connections on the socket.
    // TODO: check file system
    pub fn listen(&mut self) -> LinuxResult {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Busy => {
                let mut binding = UNIX_TABLE.write();
                let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
                socket_inner.set_state(UnixSocketStatus::Listening);
                Ok(())
            }
            _ => {
                Ok(()) //ignore simultaneous `listen`s.
            }
        }
    }

    /// Accepts a new connection from a listening socket, get info from self buffer
    pub fn accept(&mut self) -> AxResult<UnixSocket> {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Listening => {
                //buf dequeue as handle to get socket
                loop {
                    let data: &mut [u8] = &mut [0u8; core::mem::size_of::<usize>()];
                    let res = self.dequeue_buf(data);
                    match res {
                        Ok(_len) => {
                            let mut array = [0u8; core::mem::size_of::<usize>()];
                            array.copy_from_slice(data);
                            let remote_handle = usize::from_ne_bytes(array);
                            let unix_socket = UnixSocket::new(UnixSocketType::SockStream);
                            {
                                let mut binding = UNIX_TABLE.write();
                                let remote_socket = binding.get_mut(remote_handle).unwrap();
                                remote_socket
                                    .lock()
                                    .set_peersocket(unix_socket.get_sockethandle());
                            }
                            let mut binding = UNIX_TABLE.write();
                            let mut socket_inner = binding
                                .get_mut(unix_socket.get_sockethandle())
                                .unwrap()
                                .lock();
                            socket_inner.set_peersocket(remote_handle);
                            socket_inner.set_state(UnixSocketStatus::Connected);
                            return Ok(unix_socket);
                        }
                        Err(AxError::WouldBlock) => {
                            if self.is_nonblocking() {
                                return Err(AxError::WouldBlock);
                            } else {
                                yield_now();
                            }
                        }
                        Err(e) => {
                            return Err(e);
                        }
                    }
                }
            }
            _ => ax_err!(InvalidInput, "socket accept() failed: not listen"),
        }
    }

    /// Shuts down the socket.
    pub fn shutdown(&self) -> LinuxResult {
        let mut binding = UNIX_TABLE.write();
        let mut socket_inner = binding.get_mut(self.get_sockethandle()).unwrap().lock();
        socket_inner.set_state(UnixSocketStatus::Closed);
        Ok(())
    }

    /// Returns whether this socket is in nonblocking mode.
    #[inline]
    pub fn is_nonblocking(&self) -> bool {
        self.nonblock.load(Ordering::Acquire)
    }

    /// Sets the nonblocking mode for the socket.
    pub fn set_nonblocking(&self, nonblocking: bool) {
        self.nonblock.store(nonblocking, Ordering::Release);
    }

    /// Checks if the socket is in a listening state.
    pub fn is_listening(&self) -> bool {
        let now_state = self.get_state();
        match now_state {
            UnixSocketStatus::Listening => true,
            _ => false,
        }
    }

    /// Returns the socket type of the `UnixSocket`.
    pub fn get_sockettype(&self) -> UnixSocketType {
        self.unixsocket_type
    }
}

impl Drop for UnixSocket {
    fn drop(&mut self) {
        let _ = self.shutdown();
        UNIX_TABLE.write().remove(self.get_sockethandle());
    }
}

/// Initializes the global UNIX socket table, `UNIX_TABLE`, for managing Unix domain sockets.
pub(crate) fn init_unix() {
    UNIX_TABLE.init_by(RwLock::new(HashMapWarpper::new()));
}
