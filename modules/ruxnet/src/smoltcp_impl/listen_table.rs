/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/

use alloc::string::String;
use alloc::{boxed::Box, collections::VecDeque};
use core::ops::{Deref, DerefMut};

use axerrno::{ax_err, AxError, AxResult};
use axsync::Mutex;
use smoltcp::iface::{SocketHandle, SocketSet};
use smoltcp::socket::tcp::{self, State};
use smoltcp::wire::{IpAddress, IpEndpoint, IpListenEndpoint};

use super::{route_dev, to_static_str, SocketSetWrapper, ETH0, LISTEN_QUEUE_SIZE, LO, SOCKET_SET};

const PORT_NUM: usize = 65536;

struct ListenTableEntry {
    listen_endpoint: IpListenEndpoint,
    syn_queue: VecDeque<(SocketHandle, String)>,
}

impl ListenTableEntry {
    pub fn new(listen_endpoint: IpListenEndpoint) -> Self {
        Self {
            listen_endpoint,
            syn_queue: VecDeque::with_capacity(LISTEN_QUEUE_SIZE),
        }
    }

    #[inline]
    fn can_accept(&self, dst: IpAddress) -> bool {
        match self.listen_endpoint.addr {
            Some(addr) => addr == dst,
            None => true,
        }
    }
}

impl Drop for ListenTableEntry {
    fn drop(&mut self) {
        for handle in &self.syn_queue {
            SOCKET_SET.remove(handle.0, handle.1.clone());
        }
    }
}

pub struct ListenTable {
    tcp: Box<[Mutex<Option<Box<ListenTableEntry>>>]>,
}

impl ListenTable {
    pub fn new() -> Self {
        let tcp = unsafe {
            let mut buf = Box::new_uninit_slice(PORT_NUM);
            for i in 0..PORT_NUM {
                buf[i].write(Mutex::new(None));
            }
            buf.assume_init()
        };
        Self { tcp }
    }

    pub fn can_listen(&self, port: u16) -> bool {
        self.tcp[port as usize].lock().is_none()
    }

    pub fn listen(&self, listen_endpoint: IpListenEndpoint) -> AxResult {
        let port = listen_endpoint.port;
        assert_ne!(port, 0);
        let mut entry = self.tcp[port as usize].lock();
        if entry.is_none() {
            *entry = Some(Box::new(ListenTableEntry::new(listen_endpoint)));
            Ok(())
        } else {
            ax_err!(AddrInUse, "socket listen() failed")
        }
    }

    pub fn unlisten(&self, port: u16) {
        debug!("TCP socket unlisten on {}", port);
        *self.tcp[port as usize].lock() = None;
    }

    pub fn can_accept(&self, port: u16) -> AxResult<bool> {
        if let Some(entry) = self.tcp[port as usize].lock().deref() {
            Ok(entry
                .syn_queue
                .iter()
                .any(|handle| is_connected(handle.0, handle.1.clone())))
        } else {
            ax_err!(InvalidInput, "socket accept() failed: not listen")
        }
    }

    pub fn accept(&self, port: u16) -> AxResult<(SocketHandle, (IpEndpoint, IpEndpoint))> {
        if let Some(entry) = self.tcp[port as usize].lock().deref_mut() {
            let syn_queue = &mut entry.syn_queue;
            let (idx, addr_tuple) = syn_queue
                .iter()
                .enumerate()
                .find_map(|(idx, handle)| {
                    is_connected(handle.0, handle.1.clone())
                        .then(|| (idx, get_addr_tuple(handle.0, handle.1.clone())))
                })
                .ok_or(AxError::WouldBlock)?; // wait for connection
            if idx > 0 {
                warn!(
                    "slow SYN queue enumeration: index = {}, len = {}!",
                    idx,
                    syn_queue.len()
                );
            }
            let handle = syn_queue.swap_remove_front(idx).unwrap();
            Ok((handle.0, addr_tuple))
        } else {
            ax_err!(InvalidInput, "socket accept() failed: not listen")
        }
    }

    pub fn incoming_tcp_packet(
        &self,
        src: IpEndpoint,
        dst: IpEndpoint,
        sockets: &mut SocketSet<'_>,
    ) {
        if let Some(entry) = self.tcp[dst.port as usize].lock().deref_mut() {
            if !entry.can_accept(dst.addr) {
                // not listening on this address
                return;
            }
            if entry.syn_queue.len() >= LISTEN_QUEUE_SIZE {
                // SYN queue is full, drop the packet
                warn!("SYN queue overflow!");
                return;
            }
            let mut socket = SocketSetWrapper::new_tcp_socket();
            if socket.listen(entry.listen_endpoint).is_ok() {
                let handle = sockets.add(socket);
                debug!(
                    "TCP socket {}: prepare for connection {} -> {}",
                    handle, src, entry.listen_endpoint
                );
                let iface_name = match dst.addr {
                    IpAddress::Ipv4(addr) => route_dev(addr.0),
                    _ => panic!("IPv6 not supported"),
                };
                entry.syn_queue.push_back((handle, iface_name));
            }
        }
    }
}

fn is_connected(handle: SocketHandle, iface_name: String) -> bool {
    SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, iface_name, |socket| {
        !matches!(socket.state(), State::Listen | State::SynReceived)
    })
}

fn get_addr_tuple(handle: SocketHandle, iface_name: String) -> (IpEndpoint, IpEndpoint) {
    SOCKET_SET.with_socket::<tcp::Socket, _, _>(handle, iface_name, |socket| {
        (
            socket.local_endpoint().unwrap(),
            socket.remote_endpoint().unwrap(),
        )
    })
}
