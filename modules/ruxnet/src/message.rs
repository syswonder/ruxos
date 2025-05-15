/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
//! Message Queue
use alloc::{collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use axerrno::LinuxResult;
use iovec::{IoVecsInput, IoVecsOutput};
use ruxfdtable::FileLike;
use ruxfs::OpenFlags;
use ruxtask::fs::{add_file_like, get_file_like};

use crate::address::SocketAddress;

bitflags::bitflags! {
    #[repr(C)]
    #[derive(Default, Copy, Clone, Debug)]
    /// Flags used for send/recv operations on sockets.
    /// The definition is from https://elixir.bootlin.com/linux/v6.0.9/source/include/linux/socket.h
    pub struct MessageFlags: i32 {
        /// Peek at incoming messages without removing them from the queue
        const MSG_PEEK	= 2;
        /// Control data was truncated due to insufficient buffer space
        const MSG_CTRUNC	= 8;
        /// Normal data was truncated because the datagram was larger than the buffer
        const MSG_TRUNC	= 0x20;
        /// **Temporarily** Enable non-blocking operation
        const MSG_DONTWAIT	= 0x40;
        /// TODO
        const MSG_EOR = 0x80;
        /// TODO
        const MSG_WAITALL = 0x100;
        /// TODO
        const MSG_FIN = 0x200;
        /// TODO
        const MSG_SYN = 0x400;
        /// TODO
        const MSG_CONFIRM = 0x800;
        /// TODO
        const MSG_RST = 0x1000;
        /// TODO
        const MSG_ERRQUEUE = 0x2000;
        /// TODO
        const MSG_NOSIGNAL = 0x4000;
        /// TODO
        const MSG_MORE = 0x8000;
        /// TODO
        const MSG_WAITFORONE = 0x10000;
        /// TODO
        const MSG_BATCH = 0x40000;
        /// TODO
        const MSG_FASTOPEN = 0x20000000;
        /// Set close_on_exec for file descriptor received through SCM_RIGHTS
        const MSG_CMSG_CLOEXEC = 0x40000000;
    }
}

/// Represents a message containing data, optional address information,
/// and optional ancillary/control data.
pub struct Message {
    data: Vec<u8>,
    address: Option<SocketAddress>,
    ancillary_data: Vec<ControlMessageData>,
}

#[derive(Default)]
/// Contains information about a read operation from a message queue
pub struct MessageReadInfo {
    /// Number of bytes actually read into the user buffer.
    /// For partial reads, this will be less than both the buffer size and bytes_total.
    pub bytes_read: usize,
    /// The original total length of the message/datagram.
    /// For DGRAM sockets: When a 500-byte datagram is received but the user buffer
    /// only has 300 bytes space, bytes_read=300 while bytes_total=500.
    /// For STREAM sockets: Typically equals bytes_read since streams have no message boundaries.
    pub bytes_total: usize,
    /// Source address of the received message.
    pub address: Option<SocketAddress>,
    /// Additional control messages received with the data.
    /// Ancillary data is a sequence of `cmsghdr` structures with appended data.
    /// see <https://www.man7.org/linux/man-pages/man3/cmsg.3.html>
    pub ancillary_data: Vec<ControlMessageData>,
}

#[derive(Clone)]
/// Enum representing different types of control messages that can be sent/received
pub enum ControlMessageData {
    /// UNIX domain socket specific control messages
    Unix(UnixControlData),
    /// Internet domain socket control messages (IP-level options)
    Inet(InetControlData),
}
const SOL_SOCKET: i32 = 1; // Socket level for socket options
const SOL_IP: i32 = 0; // Internet Protocol level for IP options
const SOL_IPV6: i32 = 41; // Internet Protocol version 6 level for IPv6 options
const SCM_RIGHTS: i32 = 1; // Send or receive file descriptors
const SCM_CREDENTIALS: i32 = 2; // Send or receive UNIX credentials (user ID, group ID, process ID)
const SCM_SECURITY: i32 = 3; // Send or receive SELinux security context

impl ControlMessageData {
    /// Creates a new control message data instance from the provided level, type, and data.
    pub fn try_new(cmsg_level: i32, cmsg_type: i32, data: Vec<u8>) -> LinuxResult<Self> {
        match cmsg_level {
            SOL_SOCKET => {
                match cmsg_type {
                    SCM_RIGHTS => {
                        // Extract file descriptors from the data
                        let mut files = Vec::new();
                        for chunk in data.chunks_exact(core::mem::size_of::<i32>()) {
                            let fd = i32::from_ne_bytes(chunk.try_into().unwrap());
                            files.push(get_file_like(fd)?);
                        }
                        Ok(ControlMessageData::Unix(UnixControlData::Rights(files)))
                    }
                    SCM_CREDENTIALS => todo!(), // Todo: handle credentials
                    SCM_SECURITY => todo!(),    // Todo: handle security context
                    _ => Err(axerrno::LinuxError::EINVAL), // Unsupported control message type
                }
            }
            SOL_IP => todo!(),   // Todo: handle IP control messages
            SOL_IPV6 => todo!(), // Todo: handle IPv6 control messages
            _ => Err(axerrno::LinuxError::EINVAL),
        }
    }

    /// Parses the control message data into a tuple of (level, type, data)
    pub fn parse(self, flags: MessageFlags) -> LinuxResult<(i32, i32, Vec<u8>)> {
        match self {
            ControlMessageData::Unix(unix_control_data) => {
                match unix_control_data {
                    UnixControlData::Rights(files) => {
                        // Convert file descriptors to byte representation
                        let cloexec = flags
                            .contains(MessageFlags::MSG_CMSG_CLOEXEC)
                            .then_some(OpenFlags::O_CLOEXEC)
                            .unwrap_or_default();
                        let mut fds = Vec::with_capacity(files.len());
                        for file in files {
                            let fd = add_file_like(file, cloexec)?;
                            fds.push(fd);
                        }
                        debug!("received fds: {fds:?}");
                        // Safe conversion: i32 -> bytes (preserves endianness)
                        let mut data_bytes = Vec::with_capacity(fds.len() * 4);
                        for fd in fds {
                            data_bytes.extend_from_slice(&fd.to_ne_bytes());
                        }
                        Ok((SOL_SOCKET, SCM_RIGHTS, data_bytes))
                    }
                    UnixControlData::Credentials => todo!(),
                    UnixControlData::Security => todo!(),
                }
            }
            ControlMessageData::Inet(_) => todo!("Implement Internet control messages"),
        }
    }
}

impl ControlMessageData {
    /// Calculates the total size needed to store this control message data
    pub fn size(&self) -> usize {
        match self {
            ControlMessageData::Unix(data) => match data {
                UnixControlData::Rights(fds) => fds.len() * core::mem::size_of::<i32>(),
                UnixControlData::Credentials => todo!(), // Todo: size of credentials
                UnixControlData::Security => todo!(),    // Todo: size of security context
            },
            ControlMessageData::Inet(_) => todo!(), // Todo: size of Inet control data
        }
    }

    /// Returns the minimum buffer size needed to receive this type of control message data
    pub fn minium_size(&self) -> usize {
        match self {
            ControlMessageData::Unix(data) => match data {
                UnixControlData::Rights(_) => core::mem::size_of::<i32>(), // Minimum size for a single file descriptor
                UnixControlData::Credentials => todo!(), // Todo: minimum size of credentials
                UnixControlData::Security => todo!(),    // Todo: minimum size of security context
            },
            ControlMessageData::Inet(_) => todo!(), // Todo: minimum size of Inet control data
        }
    }
}

/// UNIX domain socket specific control messages
/// see <https://www.man7.org/linux/man-pages/man7/unix.7.html>
#[derive(Clone)]
pub enum UnixControlData {
    /// Send or receive a set of open file descriptors from another process.
    Rights(Vec<Arc<dyn FileLike>>),
    /// Todo: Send or receive UNIX credentials
    Credentials,
    /// Todo: Receive the SELinux security context (the security label) of the peer socket.
    Security,
}

#[derive(Clone)]
/// Todo: Internet socket control messages
pub struct InetControlData;

/// A queue for storing messages with capacity management
pub struct MessageQueue {
    /// The messages stored in the message queue.
    messages: VecDeque<Message>,
    /// Bytes received in message queue (not equal to `messages.len()`)
    length: usize,
    /// The maximum number of bytes that can be stored
    capacity: usize,
}

impl MessageQueue {
    /// Creates a new message queue with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: VecDeque::new(),
            length: 0,
            capacity,
        }
    }

    /// Returns the remaining available capacity in bytes
    pub fn available_capacity(&self) -> usize {
        self.capacity - self.length
    }

    /// Gets the total capacity of the queue
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Bytes received in message queue
    pub fn length(&self) -> usize {
        self.length
    }

    /// Checks if the queue is empty
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    /// Adds a message to the front of the queue
    pub fn push_front(&mut self, msg: Message) {
        self.length += msg.data.len();
        debug_assert!(self.length <= self.capacity);
        self.messages.push_front(msg);
    }

    /// Removes and returns the next message from the queue
    pub fn read_one_message(&mut self) -> Option<Message> {
        self.messages.pop_front().map(|msg| {
            self.length -= msg.data.len();
            msg
        })
    }

    /// Adds a message to the end of the queue
    pub fn write_one_message(&mut self, msg: Message) {
        self.length += msg.data.len();
        debug_assert!(self.length <= self.capacity);
        self.messages.push_back(msg);
    }

    /// Reads data from the queue for stream-oriented sockets
    ///
    /// Reads messages until there are no more messages, a message with ancillary data is
    /// encountered, or `dst` is full.
    pub fn read_stream(&mut self, dst: &mut IoVecsOutput) -> LinuxResult<MessageReadInfo> {
        let mut total_bytes_read = 0;
        let mut ancillary_data = Vec::new();
        let mut address = None;
        while let Some(mut msg) = self.read_one_message() {
            // All the message in message queue must be the same address because it is a stream socket.
            if msg.address.is_some() {
                if address.is_none() {
                    address = msg.address.clone();
                } else {
                    debug_assert!(address == msg.address);
                }
            }
            let bytes_read = dst.write(&msg.data);
            total_bytes_read += bytes_read;
            if bytes_read < msg.data.len() {
                let remain_bytes = msg.data.split_off(bytes_read);
                self.push_front(Message {
                    data: remain_bytes,
                    address: msg.address,
                    ancillary_data: msg.ancillary_data,
                });
                break;
            }
            if !msg.ancillary_data.is_empty() {
                ancillary_data = msg.ancillary_data;
                break;
            }
        }
        Ok(MessageReadInfo {
            bytes_read: total_bytes_read,
            bytes_total: total_bytes_read,
            address,
            ancillary_data,
        })
    }

    /// Peeks at data in the queue without removing it (stream version)
    pub fn peek_stream(&mut self, dst: &mut IoVecsOutput) -> LinuxResult<MessageReadInfo> {
        let mut total_bytes_read = 0;
        let mut ancillary_data = Vec::new();
        let mut address = None;
        for msg in self.messages.iter() {
            // All the message in message queue must be the same address because it is a stream socket.
            if msg.address.is_some() {
                if address.is_none() {
                    address = msg.address.clone();
                } else {
                    debug_assert!(address == msg.address);
                }
            }
            let bytes_read = dst.write(&msg.data);
            total_bytes_read += bytes_read;
            if bytes_read < msg.data.len() {
                break;
            }
            if !msg.ancillary_data.is_empty() {
                ancillary_data = msg.ancillary_data.clone();
                break;
            }
        }
        Ok(MessageReadInfo {
            bytes_read: total_bytes_read,
            bytes_total: total_bytes_read,
            address,
            ancillary_data,
        })
    }

    /// Reads a datagram from the queue
    pub fn read_dgram(&mut self, dst: &mut IoVecsOutput) -> LinuxResult<MessageReadInfo> {
        if let Some(msg) = self.read_one_message() {
            return Ok(MessageReadInfo {
                bytes_read: dst.write(&msg.data),
                bytes_total: msg.data.len(),
                address: msg.address,
                ancillary_data: msg.ancillary_data,
            });
        }
        Ok(MessageReadInfo::default())
    }

    /// Peeks at a datagram without removing it from the queue
    pub fn peek_dgram(&mut self, dst: &mut IoVecsOutput) -> LinuxResult<MessageReadInfo> {
        if let Some(msg) = self.messages.front() {
            return Ok(MessageReadInfo {
                bytes_read: dst.write(&msg.data),
                bytes_total: msg.data.len(),
                address: msg.address.clone(),
                ancillary_data: msg.ancillary_data.clone(),
            });
        }
        Ok(MessageReadInfo::default())
    }

    /// Writes data to the queue for stream-oriented sockets
    ///
    /// Using `&mut Vec<AncillaryData>` with core::mem::take avoids deep-copying Vec data via clone(),
    /// transferring ownership with zero cost via pointer swap instead.
    pub fn write_stream(
        &mut self,
        src: &IoVecsInput,
        address: Option<SocketAddress>,
        ancillary_data: &mut Vec<ControlMessageData>,
    ) -> LinuxResult<usize> {
        let actual_write = core::cmp::min(self.available_capacity(), src.total_len());
        if actual_write == 0 && src.total_len() > 0 {
            // Whether to finally return EAGAIN or block is determined by the outer function based on the flag bits
            // (e.g. MessageFlags::DONTWAIT, OpenFlags::O_NONBLOCK).
            return Err(axerrno::LinuxError::EAGAIN);
        }
        let message = Message {
            data: src.read_to_vec(actual_write),
            address,
            ancillary_data: core::mem::take(ancillary_data),
        };
        self.write_one_message(message);
        Ok(actual_write)
    }

    /// Writes a datagram to the queue
    ///     
    /// Using `&mut Vec<AncillaryData>` with core::mem::take avoids deep-copying Vec data via clone(),
    /// transferring ownership with zero cost via pointer swap instead.
    pub fn write_dgram(
        &mut self,
        src: &IoVecsInput,
        address: Option<SocketAddress>,
        ancillary_data: &mut Vec<ControlMessageData>,
    ) -> LinuxResult<usize> {
        let actual_write = src.total_len();
        if actual_write > self.capacity {
            // The socket type requires that message be sent atomically, and the size of the message to be sent made this impossible.
            return Err(axerrno::LinuxError::EMSGSIZE);
        }
        if actual_write > self.available_capacity() {
            // Whether to finally return EAGAIN or block is determined by the outer function based on the flag bits
            // (e.g. MessageFlags::DONTWAIT, OpenFlags::O_NONBLOCK).
            return Err(axerrno::LinuxError::EAGAIN);
        }
        let message = Message {
            data: src.read_to_vec(actual_write),
            address,
            ancillary_data: core::mem::take(ancillary_data),
        };
        self.write_one_message(message);
        Ok(actual_write)
    }
}
