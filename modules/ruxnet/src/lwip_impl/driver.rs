use super::LWIP_MUTEX;
use crate::{
    net_impl::addr::{mask_to_prefix, MacAddr},
    IpAddr,
};
use alloc::{boxed::Box, collections::VecDeque, sync::Arc};
#[cfg(feature = "irq")]
use axdriver::register_interrupt_handler;
use axsync::Mutex;
use core::{cell::RefCell, ffi::c_void};
use driver_net::{DevError, NetBuf, NetBufBox};
use lazy_init::LazyInit;
use lwip_rust::bindings::{
    err_enum_t_ERR_MEM, err_enum_t_ERR_OK, err_t, etharp_output, ethernet_input, ip4_addr_t,
    lwip_htonl, lwip_init, netif, netif_add, netif_poll, netif_set_default, netif_set_link_up,
    netif_set_up, pbuf, pbuf_free, rx_custom_pbuf_alloc, rx_custom_pbuf_free, rx_custom_pbuf_init,
    rx_custom_pbuf_t, sys_check_timeouts, NETIF_FLAG_BROADCAST, NETIF_FLAG_ETHARP,
    NETIF_FLAG_ETHERNET,
};
use ruxdriver::prelude::*;

const RX_BUF_QUEUE_SIZE: usize = 64;

struct NetifWrapper(netif);
unsafe impl Send for NetifWrapper {}

struct DeviceWrapper {
    inner: RefCell<AxNetDevice>, // use `RefCell` is enough since it's wrapped in `Mutex` in `InterfaceWrapper`.
    rx_buf_queue: VecDeque<NetBufBox>,
}

impl DeviceWrapper {
    fn new(inner: AxNetDevice) -> Self {
        Self {
            inner: RefCell::new(inner),
            rx_buf_queue: VecDeque::with_capacity(RX_BUF_QUEUE_SIZE),
        }
    }

    fn poll(&mut self) {
        while self.rx_buf_queue.len() < RX_BUF_QUEUE_SIZE {
            match self.inner.borrow_mut().receive() {
                Ok(bufptr) => unsafe {
                    self.rx_buf_queue.push_back(NetBuf::from_buf_ptr(bufptr));
                },
                Err(DevError::Again) => break, // TODO: better method to avoid error type conversion
                Err(err) => {
                    warn!("receive failed: {:?}", err);
                    break;
                }
            }
        }
    }

    fn receive(&mut self) -> Option<NetBufBox> {
        self.rx_buf_queue.pop_front()
    }

    #[cfg(feature = "irq")]
    fn ack_interrupt(&mut self) -> bool {
        unsafe { self.inner.as_ptr().as_mut().unwrap().ack_interrupt() }
    }
}

struct InterfaceWrapper {
    name: &'static str,
    dev: Arc<Mutex<DeviceWrapper>>,
    netif: Mutex<NetifWrapper>,
}

impl InterfaceWrapper {
    pub fn name(&self) -> &str {
        self.name
    }

    pub fn poll(&self) {
        self.dev.lock().poll();
        loop {
            let buf_receive = self.dev.lock().receive();
            if let Some(buf) = buf_receive {
                trace!("RECV {} bytes: {:02X?}", buf.packet().len(), buf.packet());

                let length = buf.packet().len();
                let payload_mem = buf.packet().as_ptr() as *mut _;
                let payload_mem_len = buf.capacity() as u16;
                let p = unsafe {
                    rx_custom_pbuf_alloc(
                        Some(pbuf_free_custom),
                        Box::into_raw(buf) as *mut _,
                        Arc::into_raw(self.dev.clone()) as *mut _,
                        length as u16,
                        payload_mem,
                        payload_mem_len,
                    )
                };

                debug!("ethernet_input");
                let mut netif = self.netif.lock();
                unsafe {
                    let res = netif.0.input.unwrap()(p, &mut netif.0);
                    if (res as i32) != err_enum_t_ERR_OK {
                        warn!("ethernet_input failed: {:?}", res);
                        pbuf_free(p);
                    }
                }
            } else {
                break;
            }
        }
    }

    #[cfg(feature = "irq")]
    pub fn ack_interrupt(&self) {
        unsafe { &mut *self.dev.as_mut_ptr() }.ack_interrupt();
    }
}

extern "C" fn pbuf_free_custom(p: *mut pbuf) {
    trace!("pbuf_free_custom: {:x?}", p);
    let p = p as *mut rx_custom_pbuf_t;
    let buf = unsafe { Box::from_raw((*p).buf as *mut NetBuf) };
    let dev = unsafe { Arc::from_raw((*p).dev as *const Mutex<DeviceWrapper>) };
    match dev
        .lock()
        .inner
        .borrow_mut()
        .recycle_rx_buffer(NetBuf::into_buf_ptr(buf))
    {
        Ok(_) => (),
        Err(err) => {
            warn!("recycle_rx_buffer failed: {:?}", err);
        }
    };
    unsafe {
        rx_custom_pbuf_free(p);
    };
}

extern "C" fn ethif_init(netif: *mut netif) -> err_t {
    trace!("ethif_init");
    unsafe {
        (*netif).name[0] = 'e' as i8;
        (*netif).name[1] = 'n' as i8;
        (*netif).num = 0;

        (*netif).output = Some(etharp_output);
        (*netif).linkoutput = Some(ethif_output);

        (*netif).mtu = 1500;
        (*netif).flags = 0;
        (*netif).flags = (NETIF_FLAG_BROADCAST | NETIF_FLAG_ETHARP | NETIF_FLAG_ETHERNET) as u8;
    }
    err_enum_t_ERR_OK as err_t
}

extern "C" fn ethif_output(netif: *mut netif, p: *mut pbuf) -> err_t {
    trace!("ethif_output");
    let ethif = unsafe { &mut *((*netif).state as *mut _ as *mut InterfaceWrapper) };
    let dev_wrapper = ethif.dev.lock();
    let mut dev = dev_wrapper.inner.borrow_mut();

    if dev.can_transmit() {
        unsafe {
            let tot_len = (*p).tot_len;
            let mut tx_buf = *NetBuf::from_buf_ptr(dev.alloc_tx_buffer(tot_len.into()).unwrap());
            dev.prepare_tx_buffer(&mut tx_buf, tot_len.into()).unwrap();

            // Copy pbuf chain to tx_buf
            let mut offset = 0;
            let mut q = p;
            while !q.is_null() {
                let len = (*q).len as usize;
                let payload = (*q).payload;
                let payload = core::slice::from_raw_parts(payload as *const u8, len);
                tx_buf.packet_mut()[offset..offset + len].copy_from_slice(payload);
                offset += len;
                q = (*q).next;
            }

            trace!(
                "SEND {} bytes: {:02X?}",
                tx_buf.packet().len(),
                tx_buf.packet()
            );
            dev.transmit(NetBuf::into_buf_ptr(Box::new(tx_buf)))
                .unwrap();
            err_enum_t_ERR_OK as err_t
        }
    } else {
        error!("[ethif_output] dev can't transmit");
        err_enum_t_ERR_MEM as err_t
    }
}

static ETH0: LazyInit<InterfaceWrapper> = LazyInit::new();

/// Poll the network stack.
///
/// It may receive packets from the NIC and process them, and transmit queued
/// packets to the NIC.
pub fn poll_interfaces() {
    ETH0.poll();
    unsafe {
        netif_poll(&mut ETH0.netif.lock().0);
    }
}

fn ip4_addr_gen(a: u8, b: u8, c: u8, d: u8) -> ip4_addr_t {
    ip4_addr_t {
        addr: unsafe {
            lwip_htonl(((a as u32) << 24) | ((b as u32) << 16) | ((c as u32) << 8) | (d as u32))
        },
    }
}
pub fn init() {}

pub fn init_netdev(net_dev: AxNetDevice) {
    match net_dev.device_name() {
        "loopback" => {
            info!("use lwip netif loopback");
        }
        _ => {
            LWIP_MUTEX.init_by(Mutex::new(0));
            let _guard = LWIP_MUTEX.lock();

            let ipaddr: ip4_addr_t = ip4_addr_gen(10, 0, 2, 15); // QEMU user networking default IP
            let netmask: ip4_addr_t = ip4_addr_gen(255, 255, 255, 0);
            let gw: ip4_addr_t = ip4_addr_gen(10, 0, 2, 2); // QEMU user networking gateway

            let dev = net_dev;
            let mut netif: netif = unsafe { core::mem::zeroed() };
            netif.hwaddr_len = 6;
            netif.hwaddr = dev.mac_address().0;

            ETH0.init_by(InterfaceWrapper {
                name: "eth0",
                dev: Arc::new(Mutex::new(DeviceWrapper::new(dev))),
                netif: Mutex::new(NetifWrapper(netif)),
            });

            unsafe {
                lwip_init();
                rx_custom_pbuf_init();
                netif_add(
                    &mut ETH0.netif.lock().0,
                    &ipaddr,
                    &netmask,
                    &gw,
                    &ETH0 as *const _ as *mut c_void,
                    Some(ethif_init),
                    Some(ethernet_input),
                );
                netif_set_link_up(&mut ETH0.netif.lock().0);
                netif_set_up(&mut ETH0.netif.lock().0);
                netif_set_default(&mut ETH0.netif.lock().0);
            }

            info!("created net interface {:?}:", ETH0.name());
            info!(
                "  ether:    {}",
                MacAddr::from_bytes(&ETH0.netif.lock().0.hwaddr)
            );
            let ip = IpAddr::from(ETH0.netif.lock().0.ip_addr);
            let mask = mask_to_prefix(IpAddr::from(ETH0.netif.lock().0.netmask)).unwrap();
            info!("  ip:       {}/{}", ip, mask);
            info!("  gateway:  {}", IpAddr::from(ETH0.netif.lock().0.gw));
        }
    }
}

pub fn lwip_loop_once() {
    let guard = LWIP_MUTEX.lock();
    unsafe {
        ETH0.poll();
        netif_poll(&mut ETH0.netif.lock().0);
        sys_check_timeouts();
    }
    drop(guard);
}
