/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */
use axsync::Mutex;
use core::net::{IpAddr, Ipv4Addr, SocketAddr};
use driver_9p::_9pDriverOps;
use driver_common::{BaseDriverOps, DeviceType};
use log::*;
use ruxnet::{message::MessageFlags, TcpSocket};

pub struct Net9pDev {
    socket: Mutex<TcpSocket>,
    srv_addr: SocketAddr,
}

impl Net9pDev {
    pub fn new(ip: &[u8], port: u16) -> Self {
        let ip_addr = match ip.len() {
            4 => IpAddr::V4(Ipv4Addr::new(ip[0], ip[1], ip[2], ip[3])),
            _ => {
                error!("Unsupport IP address: {ip:?}, using 0.0.0.0 instead");
                IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0))
            }
        };
        Self {
            socket: Mutex::new(TcpSocket::new(false)),
            srv_addr: SocketAddr::new(ip_addr, port),
        }
    }
}

impl BaseDriverOps for Net9pDev {
    fn device_name(&self) -> &str {
        "net-9p"
    }

    fn device_type(&self) -> DeviceType {
        DeviceType::_9P
    }
}

impl _9pDriverOps for Net9pDev {
    // initialize self(e.g. setup TCP connection)
    fn init(&self) -> Result<(), u8> {
        info!("9P client connecting to {:?}", self.srv_addr);
        match self.socket.lock().connect(self.srv_addr) {
            Ok(_) => {
                info!("net9p connected successfully");
                Ok(())
            }
            Err(_) => {
                error!("net9p connected failed");
                Err(0)
            }
        }
    }

    // send bytes of inputs as request and receive  get answer in outputs
    fn send_with_recv(&mut self, inputs: &[u8], outputs: &mut [u8]) -> Result<u32, u8> {
        match self.socket.lock().send(inputs) {
            Ok(length) => {
                debug!("net9p send successfully,length = {length}");
            }
            Err(_) => {
                error!("net9p send failed");
                return Err(0);
            }
        }
        match self.socket.lock().recv(outputs, MessageFlags::empty()) {
            Ok(length) => {
                debug!("net9p recv successfully,length = {length}");
                Ok(length as u32)
            }
            Err(_) => {
                error!("net9p recv failed");
                Err(0)
            }
        }
    }
}
