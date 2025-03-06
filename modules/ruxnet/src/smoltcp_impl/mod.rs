/* Copyright (c) [2023] [Syswonder Community]
 *   [Ruxos] is licensed under Mulan PSL v2.
 *   You can use this software according to the terms and conditions of the Mulan PSL v2.
 *   You may obtain a copy of Mulan PSL v2 at:
 *               http://license.coscl.org.cn/MulanPSL2
 *   THIS SOFTWARE IS PROVIDED ON AN "AS IS" BASIS, WITHOUT WARRANTIES OF ANY KIND, EITHER EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO NON-INFRINGEMENT, MERCHANTABILITY OR FIT FOR A PARTICULAR PURPOSE.
 *   See the Mulan PSL v2 for more details.
 */

mod addr;
mod bench;
mod dns;
mod listen_table;
mod tcp;
mod udp;

use alloc::string::{String, ToString};
use alloc::vec;
use core::cell::RefCell;
use core::ops::DerefMut;

use axsync::Mutex;
use driver_net::{DevError, NetBufPtr};
use lazy_init::LazyInit;
use ruxdriver::prelude::*;
use ruxhal::time::{current_time_nanos, NANOS_PER_MICROS};
use smoltcp::iface::{Config, Interface, SocketHandle, SocketSet};
use smoltcp::phy::{Device, DeviceCapabilities, Medium, RxToken, TxToken};
use smoltcp::socket::{self, AnySocket};
use smoltcp::time::Instant;
use smoltcp::wire::{EthernetAddress, HardwareAddress, IpAddress, IpCidr};

use self::listen_table::ListenTable;

pub use self::dns::dns_query;
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;

macro_rules! env_or_default {
    ($key:literal) => {
        match option_env!($key) {
            Some(val) => val,
            None => "",
        }
    };
}

const IP: &str = env_or_default!("RUX_IP");
const GATEWAY: &str = env_or_default!("RUX_GW");
const DNS_SEVER: &str = "8.8.8.8";
const IP_PREFIX: u8 = 24;

const STANDARD_MTU: usize = 1500;

const RANDOM_SEED: u64 = 0xA2CE_05A2_CE05_A2CE;

const TCP_RX_BUF_LEN: usize = 64 * 1024;
const TCP_TX_BUF_LEN: usize = 64 * 1024;
const UDP_RX_BUF_LEN: usize = 64 * 1024;
const UDP_TX_BUF_LEN: usize = 64 * 1024;
const LISTEN_QUEUE_SIZE: usize = 512;

static LISTEN_TABLE: LazyInit<ListenTable> = LazyInit::new();
static SOCKET_SET: LazyInit<SocketSetWrapper> = LazyInit::new();
static IFACE_LIST: LazyInit<Mutex<vec::Vec<InterfaceWrapper>>> = LazyInit::new();

fn route_dev(addr: [u8; 4]) -> String {
    if addr[0] == 127 {
        "loopback".to_string()
    } else {
        "eth0".to_string()
    }
}

struct SocketSetWrapper<'a>(Mutex<SocketSet<'a>>);

struct DeviceWrapper {
    inner: RefCell<AxNetDevice>, // use `RefCell` is enough since it's wrapped in `Mutex` in `InterfaceWrapper`.
}

struct InterfaceWrapper {
    name: &'static str,
    ether_addr: EthernetAddress,
    dev: Mutex<DeviceWrapper>,
    iface: Mutex<Interface>,
}

impl<'a> SocketSetWrapper<'a> {
    fn new() -> Self {
        Self(Mutex::new(SocketSet::new(vec![])))
    }

    pub fn new_tcp_socket() -> socket::tcp::Socket<'a> {
        let tcp_rx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_RX_BUF_LEN]);
        let tcp_tx_buffer = socket::tcp::SocketBuffer::new(vec![0; TCP_TX_BUF_LEN]);
        socket::tcp::Socket::new(tcp_rx_buffer, tcp_tx_buffer)
    }

    pub fn new_udp_socket() -> socket::udp::Socket<'a> {
        let udp_rx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_RX_BUF_LEN],
        );
        let udp_tx_buffer = socket::udp::PacketBuffer::new(
            vec![socket::udp::PacketMetadata::EMPTY; 8],
            vec![0; UDP_TX_BUF_LEN],
        );
        socket::udp::Socket::new(udp_rx_buffer, udp_tx_buffer)
    }

    pub fn new_dns_socket() -> socket::dns::Socket<'a> {
        let server_addr = DNS_SEVER.parse().expect("invalid DNS server address");
        socket::dns::Socket::new(&[server_addr], vec![])
    }

    pub fn add<T: AnySocket<'a>>(&self, socket: T) -> SocketHandle {
        let handle = self.0.lock().add(socket);
        debug!("socket {}: created", handle);
        handle
    }

    pub fn with_socket<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&T) -> R,
    {
        let set = self.0.lock();
        let socket = set.get(handle);
        f(socket)
    }

    pub fn with_socket_mut<T: AnySocket<'a>, R, F>(&self, handle: SocketHandle, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut set = self.0.lock();
        let socket = set.get_mut(handle);
        f(socket)
    }

    pub fn poll_interfaces(&self, iface_name: Option<String>) {
        for iface in IFACE_LIST.lock().iter() {
            if iface_name.is_none() || iface_name.clone().unwrap() == iface.name() {
                iface.poll(&self.0);
            }
        }
    }

    pub fn remove(&self, handle: SocketHandle) {
        self.0.lock().remove(handle);
        debug!("socket {}: destroyed", handle);
    }
}

impl InterfaceWrapper {
    fn new(name: &'static str, dev: AxNetDevice, ether_addr: EthernetAddress) -> Self {
        let mut config = Config::new(HardwareAddress::Ethernet(ether_addr));
        config.random_seed = RANDOM_SEED;

        let mut dev = DeviceWrapper::new(dev);
        let iface = Mutex::new(Interface::new(config, &mut dev, Self::current_time()));
        Self {
            name,
            ether_addr,
            dev: Mutex::new(dev),
            iface,
        }
    }

    fn current_time() -> Instant {
        Instant::from_micros_const((current_time_nanos() / NANOS_PER_MICROS) as i64)
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ether_addr
    }

    pub fn setup_ip_addr(&self, ip: IpAddress, prefix_len: u8) {
        let mut iface = self.iface.lock();
        iface.update_ip_addrs(|ip_addrs| {
            ip_addrs.push(IpCidr::new(ip, prefix_len)).unwrap();
        });
    }

    pub fn setup_gateway(&self, gateway: IpAddress) {
        let mut iface = self.iface.lock();
        match gateway {
            IpAddress::Ipv4(v4) => iface.routes_mut().add_default_ipv4_route(v4).unwrap(),
        };
    }

    pub fn poll(&self, sockets: &Mutex<SocketSet>) {
        let mut dev = self.dev.lock();
        let mut iface = self.iface.lock();
        let mut sockets = sockets.lock();
        let timestamp = Self::current_time();
        iface.poll(timestamp, dev.deref_mut(), &mut sockets);
    }
}

impl DeviceWrapper {
    fn new(inner: AxNetDevice) -> Self {
        Self {
            inner: RefCell::new(inner),
        }
    }
}

impl Device for DeviceWrapper {
    type RxToken<'a> = AxNetRxToken<'a> where Self: 'a;
    type TxToken<'a> = AxNetTxToken<'a> where Self: 'a;

    fn receive(&mut self, _timestamp: Instant) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let mut dev = self.inner.borrow_mut();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }

        if !dev.can_transmit() {
            return None;
        }
        let rx_buf = match dev.receive() {
            Ok(buf) => buf,
            Err(err) => {
                if !matches!(err, DevError::Again) {
                    warn!("receive failed: {:?}", err);
                }
                return None;
            }
        };
        Some((AxNetRxToken(&self.inner, rx_buf), AxNetTxToken(&self.inner)))
    }

    fn transmit(&mut self, _timestamp: Instant) -> Option<Self::TxToken<'_>> {
        let mut dev = self.inner.borrow_mut();
        if let Err(e) = dev.recycle_tx_buffers() {
            warn!("recycle_tx_buffers failed: {:?}", e);
            return None;
        }
        if dev.can_transmit() {
            Some(AxNetTxToken(&self.inner))
        } else {
            None
        }
    }

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = 1514;
        caps.max_burst_size = None;
        caps.medium = Medium::Ethernet;
        caps
    }
}

struct AxNetRxToken<'a>(&'a RefCell<AxNetDevice>, NetBufPtr);
struct AxNetTxToken<'a>(&'a RefCell<AxNetDevice>);

impl<'a> RxToken for AxNetRxToken<'a> {
    fn preprocess(&self, sockets: &mut SocketSet<'_>) {
        snoop_tcp_packet(self.1.packet(), sockets).ok();
    }

    fn consume<R, F>(self, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut rx_buf = self.1;
        trace!(
            "RECV {} bytes: {:02X?}",
            rx_buf.packet_len(),
            rx_buf.packet()
        );
        let result = f(rx_buf.packet_mut());
        self.0.borrow_mut().recycle_rx_buffer(rx_buf).unwrap();
        result
    }
}

impl<'a> TxToken for AxNetTxToken<'a> {
    fn consume<R, F>(self, len: usize, f: F) -> R
    where
        F: FnOnce(&mut [u8]) -> R,
    {
        let mut dev = self.0.borrow_mut();
        let mut tx_buf = dev.alloc_tx_buffer(len).unwrap();
        let ret = f(tx_buf.packet_mut());
        dev.transmit(tx_buf).unwrap();
        ret
    }
}

fn snoop_tcp_packet(buf: &[u8], sockets: &mut SocketSet<'_>) -> Result<(), smoltcp::wire::Error> {
    use smoltcp::wire::{EthernetFrame, IpProtocol, Ipv4Packet, TcpPacket};

    let ether_frame = EthernetFrame::new_checked(buf)?;
    let ipv4_packet = Ipv4Packet::new_checked(ether_frame.payload())?;

    if ipv4_packet.next_header() == IpProtocol::Tcp {
        let tcp_packet = TcpPacket::new_checked(ipv4_packet.payload())?;
        let src_addr = (ipv4_packet.src_addr(), tcp_packet.src_port()).into();
        let dst_addr = (ipv4_packet.dst_addr(), tcp_packet.dst_port()).into();
        let is_first = tcp_packet.syn() && !tcp_packet.ack();
        if is_first {
            // create a socket for the first incoming TCP packet, as the later accept() returns.
            LISTEN_TABLE.incoming_tcp_packet(src_addr, dst_addr, sockets);
        }
    }
    Ok(())
}

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() {
    SOCKET_SET.poll_interfaces(None);
}

/// Benchmark raw socket transmit bandwidth.
pub fn bench_transmit() {
    IFACE_LIST
        .lock()
        .iter()
        .find(|iface| iface.name() == "eth0")
        .unwrap()
        .dev
        .lock()
        .bench_transmit_bandwidth();
}

/// Benchmark raw socket receive bandwidth.
pub fn bench_receive() {
    IFACE_LIST
        .lock()
        .iter()
        .find(|iface| iface.name() == "eth0")
        .unwrap()
        .dev
        .lock()
        .bench_receive_bandwidth();
}

pub(crate) fn init() {
    let socketset = SocketSetWrapper::new();

    IFACE_LIST.init_by(Mutex::new(vec::Vec::new()));
    SOCKET_SET.init_by(socketset);
    LISTEN_TABLE.init_by(ListenTable::new());
}

pub(crate) fn init_netdev(net_dev: AxNetDevice) {
    match net_dev.device_name() {
        "loopback" => {
            let ether_addr = EthernetAddress(net_dev.mac_address().0);
            let lo = InterfaceWrapper::new("loopback", net_dev, ether_addr);

            let ip = "127.0.0.1".parse().expect("invalid IP address");
            lo.setup_ip_addr(ip, IP_PREFIX);

            info!("created net interface {:?}:", lo.name());
            info!("  ether:    {}", lo.ethernet_address());
            info!("  ip:       {}/{}", "127.0.0.1", IP_PREFIX);
            IFACE_LIST.lock().push(lo);
        }
        _ => {
            let ether_addr = EthernetAddress(net_dev.mac_address().0);
            let eth0 = InterfaceWrapper::new("eth0", net_dev, ether_addr);

            let ip = IP.parse().expect("invalid IP address");
            let gateway = GATEWAY.parse().expect("invalid gateway IP address");
            eth0.setup_ip_addr(ip, IP_PREFIX);
            eth0.setup_gateway(gateway);

            info!("created net interface {:?}:", eth0.name());
            info!("  ether:    {}", eth0.ethernet_address());
            info!("  ip:       {}/{}", ip, IP_PREFIX);
            info!("  gateway:  {}", gateway);

            IFACE_LIST.lock().push(eth0);
        }
    }
}
