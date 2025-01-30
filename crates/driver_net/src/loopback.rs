/* Copyright (c) [2023] [Syswonder Community]
*   [Ruxos] is licensed under Mulan PSL v2.
*   You can use this software according to the terms and conditions of the Mulan PSL v2.
*   You may obtain a copy of Mulan PSL v2 at:
*               http://license.coscl.org.cn/MulanPSL2
*   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
*   See the Mulan PSL v2 for more details.
*/
use crate::{EthernetAddress, NetBuf, NetBufBox, NetBufPool, NetBufPtr, NetDriverOps};
use alloc::collections::VecDeque;
use alloc::sync::Arc;
use driver_common::{BaseDriverOps, DevError, DevResult, DeviceType};

extern crate alloc;

const NET_BUF_LEN: usize = 1526;

/// The VirtIO network device driver.
///
/// `QS` is the VirtIO queue size.
pub struct LoopbackDevice {
    mac_address: EthernetAddress,
    pub(crate) queue: VecDeque<NetBufBox>,
    buf_pool: Arc<NetBufPool>,
}

unsafe impl Send for LoopbackDevice {}
unsafe impl Sync for LoopbackDevice {}

impl LoopbackDevice {
    /// Creates a new driver instance and initializes the device
    pub fn new(mac_address: Option<[u8; 6]>) -> Self {
        let buf_pool = match NetBufPool::new(1024, NET_BUF_LEN) {
            Ok(pool) => pool,
            Err(_) => {
                panic!("fail to create netbufpool");
            }
        };
        Self {
            mac_address: match mac_address {
                Some(address) => EthernetAddress(address),
                None => EthernetAddress([0; 6]),
            },
            queue: VecDeque::new(),
            buf_pool,
        }
    }
}

impl BaseDriverOps for LoopbackDevice {
    fn device_name(&self) -> &str {
        "loopback"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::Net
    }
}

impl NetDriverOps for LoopbackDevice {
    #[inline]
    fn mac_address(&self) -> EthernetAddress {
        EthernetAddress(self.mac_address.0)
    }

    #[inline]
    fn can_transmit(&self) -> bool {
        true
    }

    #[inline]
    fn can_receive(&self) -> bool {
        !self.queue.is_empty()
    }

    #[inline]
    fn rx_queue_size(&self) -> usize {
        self.queue.len()
    }

    #[inline]
    fn tx_queue_size(&self) -> usize {
        self.queue.len()
    }

    fn fill_rx_buffers(&mut self, _buf_pool: &Arc<NetBufPool>) -> DevResult {
        Ok(())
    }

    fn recycle_rx_buffer(&mut self, _rx_buf: NetBufPtr) -> DevResult {
        Ok(())
    }

    fn recycle_tx_buffers(&mut self) -> DevResult {
        Ok(())
    }

    fn prepare_tx_buffer(&self, _tx_buf: &mut NetBuf, _pkt_len: usize) -> DevResult {
        Ok(())
    }

    fn transmit(&mut self, tx_buf: NetBufPtr) -> DevResult {
        unsafe { self.queue.push_back(NetBuf::from_buf_ptr(tx_buf)) }
        Ok(())
    }

    fn receive(&mut self) -> DevResult<NetBufPtr> {
        if let Some(token) = self.queue.pop_front() {
            Ok(token.into_buf_ptr())
        } else {
            Err(DevError::Again)
        }
    }

    fn alloc_tx_buffer(&mut self, size: usize) -> DevResult<NetBufPtr> {
        let mut net_buf = self.buf_pool.alloc_boxed().ok_or(DevError::NoMemory)?;
        let pkt_len = size;

        // 1. Check if the buffer is large enough.
        let hdr_len = net_buf.header_len();
        if hdr_len + pkt_len > net_buf.capacity() {
            return Err(DevError::InvalidParam);
        }
        net_buf.set_packet_len(pkt_len);

        // 2. Return the buffer.
        Ok(net_buf.into_buf_ptr())
    }
}
