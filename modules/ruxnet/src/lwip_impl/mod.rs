mod addr;
mod dns;
mod driver;
mod tcp;
mod udp;

pub use self::addr::{IpAddr, Ipv4Addr, SocketAddr};
pub use self::dns::{dns_query, resolve_socket_addr};
pub use self::driver::{init, poll_interfaces};
pub use self::tcp::TcpSocket;
pub use self::udp::UdpSocket;
use core::ffi::c_uint;
use ruxhal::time::current_time;

use axsync::Mutex;
use lazy_init::LazyInit;

static LWIP_MUTEX: LazyInit<Mutex<u32>> = LazyInit::new();

const RECV_QUEUE_LEN: usize = 16;
const ACCEPT_QUEUE_LEN: usize = 16;

#[no_mangle]
extern "C" fn sys_now() -> c_uint {
    current_time().as_millis() as c_uint
}
