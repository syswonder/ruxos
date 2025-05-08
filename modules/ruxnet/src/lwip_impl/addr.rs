use core::{
    fmt::{self, Error},
    str::FromStr,
};

use lwip_rust::bindings::ip_addr_t;

use core::net::{Ipv4Addr as CoreIpv4Addr, SocketAddr as CoreSocketAddr, SocketAddrV4};

/// Mac Address
#[derive(Clone, Copy, Debug, Default)]
pub struct MacAddr(pub [u8; 6]);

/// IP Address, either IPv4 or IPv6
#[derive(Clone, Copy, Debug)]
pub enum IpAddr {
    /// IPv4 Address
    Ipv4(Ipv4Addr),

    /// IPv6 Address
    Ipv6(Ipv6Addr),
}

/// IPv4 Address (host byte order)
#[derive(Clone, Copy, Debug, Default)]
pub struct Ipv4Addr(pub [u8; 4]);

/// IPv6 Address
#[derive(Clone, Copy, Debug, Default)]
pub struct Ipv6Addr {
    /// Address in host byte order
    pub addr: [u32; 4usize],

    /// Zone identifier
    pub zone: u8,
}

/// Socket Address (IP Address + Port)
#[derive(Clone, Copy, Debug)]
pub struct SocketAddr {
    /// IP Address
    pub addr: IpAddr,

    /// Port
    pub port: u16,
}

impl MacAddr {
    /// Create a new MacAddr from a byte array
    pub fn from_bytes(bytes: &[u8]) -> MacAddr {
        let mut addr = [0u8; 6];
        addr.copy_from_slice(bytes);
        MacAddr(addr)
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}-{:02x}-{:02x}-{:02x}-{:02x}-{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5]
        )
    }
}

impl IpAddr {
    /// Get the IP Address as a byte array
    pub fn as_bytes(&self) -> &[u8] {
        match self {
            IpAddr::Ipv4(Ipv4Addr(addr)) => &addr[..],
            _ => panic!("IPv6 not supported"),
        }
    }
}

impl From<Ipv4Addr> for IpAddr {
    fn from(addr: Ipv4Addr) -> IpAddr {
        IpAddr::Ipv4(addr)
    }
}

impl From<Ipv6Addr> for IpAddr {
    fn from(addr: Ipv6Addr) -> IpAddr {
        IpAddr::Ipv6(addr)
    }
}

impl FromStr for IpAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<IpAddr, ()> {
        let mut parts = s.split('.');
        let mut addr: u32 = 0;
        for i in 0..4 {
            let part = parts.next().ok_or(())?;
            let part = part.parse::<u8>().map_err(|_| ())?;
            addr |= (part as u32) << (8 * i);
        }
        Ok(IpAddr::Ipv4(Ipv4Addr(addr.to_be_bytes())))
    }
}

impl From<IpAddr> for ip_addr_t {
    fn from(val: IpAddr) -> Self {
        match val {
            IpAddr::Ipv4(Ipv4Addr(addr)) => ip_addr_t {
                addr: Ipv4Addr(addr).to_u32(),
            },
            _ => panic!("IPv6 not supported"),
        }
    }
}

impl From<ip_addr_t> for IpAddr {
    #[allow(non_upper_case_globals)]
    fn from(addr: ip_addr_t) -> IpAddr {
        IpAddr::Ipv4(Ipv4Addr(addr.addr.to_be_bytes()))
    }
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IpAddr::Ipv4(ipv4_addr) => write!(f, "{ipv4_addr}"),
            _ => panic!("IPv6 not supported"),
        }
    }
}

impl Ipv4Addr {
    /// Construct an IPv4 address from parts.
    pub fn new(a0: u8, a1: u8, a2: u8, a3: u8) -> Ipv4Addr {
        Self::from_bytes(&[a0, a1, a2, a3])
    }

    /// Create a new Ipv4Addr from a byte array
    pub fn from_bytes(bytes: &[u8]) -> Ipv4Addr {
        let mut addr: u32 = 0;
        for (i, &b) in bytes.iter().enumerate().take(4) {
            addr |= (b as u32) << (8 * i);
        }
        Ipv4Addr(addr.to_be_bytes())
    }

    ///convert Ipv4Addr to u32
    pub fn to_u32(&self) -> u32 {
        let mut addr: u32 = 0;
        for (i, &b) in self.0.iter().enumerate().take(4) {
            addr |= (b as u32) << (8 * i);
        }
        addr
    }
}

impl fmt::Display for Ipv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let bytes = self.to_u32();
        write!(
            f,
            "{}.{}.{}.{}",
            bytes & 0xff,
            (bytes >> 8) & 0xff,
            (bytes >> 16) & 0xff,
            (bytes >> 24) & 0xff
        )
    }
}

// Reference: https://github.com/smoltcp-rs/smoltcp/blob/9027825c16c9c3fbadb7663e56d64b590fc95d5a/src/wire/ipv6.rs#L247-L306
// Modified to use [u32; 4] instead of [u8; 16]
impl fmt::Display for Ipv6Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // The string representation of an IPv6 address should
        // collapse a series of 16 bit sections that evaluate
        // to 0 to "::"
        //
        // See https://tools.ietf.org/html/rfc4291#section-2.2
        // for details.
        enum State {
            Head,
            HeadBody,
            Tail,
            TailBody,
        }
        let mut words = [0u16; 8];
        for i in 0..4 {
            words[i * 2] = ((self.addr[i] & 0xffff) as u16).swap_bytes();
            words[i * 2 + 1] = (((self.addr[i] >> 16) & 0xffff) as u16).swap_bytes();
        }
        let mut state = State::Head;
        for word in words.iter() {
            state = match (*word, &state) {
                // Once a u16 equal to zero write a double colon and
                // skip to the next non-zero u16.
                (0, &State::Head) | (0, &State::HeadBody) => {
                    write!(f, "::")?;
                    State::Tail
                }
                // Continue iterating without writing any characters until
                // we hit a non-zero value.
                (0, &State::Tail) => State::Tail,
                // When the state is Head or Tail write a u16 in hexadecimal
                // without the leading colon if the value is not 0.
                (_, &State::Head) => {
                    write!(f, "{word:x}")?;
                    State::HeadBody
                }
                (_, &State::Tail) => {
                    write!(f, "{word:x}")?;
                    State::TailBody
                }
                // Write the u16 with a leading colon when parsing a value
                // that isn't the first in a section
                (_, &State::HeadBody) | (_, &State::TailBody) => {
                    write!(f, ":{word:x}")?;
                    state
                }
            }
        }
        Ok(())
    }
}

impl From<SocketAddr> for CoreSocketAddr {
    fn from(socket_addr: SocketAddr) -> Self {
        match socket_addr.addr {
            IpAddr::Ipv4(ipv4_addr) => {
                let octets = ipv4_addr.0;
                let std_ipv4 = CoreIpv4Addr::new(octets[0], octets[1], octets[2], octets[3]);
                CoreSocketAddr::V4(SocketAddrV4::new(std_ipv4, socket_addr.port))
            }
            _ => panic!("IPv6 not supported"),
        }
    }
}

impl From<CoreSocketAddr> for SocketAddr {
    fn from(std_socket_addr: CoreSocketAddr) -> Self {
        match std_socket_addr {
            CoreSocketAddr::V4(v4_addr) => {
                let octets = v4_addr.ip().octets();
                let ipv4_addr = Ipv4Addr(u32::from_be_bytes(octets).to_be_bytes());
                SocketAddr {
                    addr: IpAddr::Ipv4(ipv4_addr),
                    port: v4_addr.port(),
                }
            }
            _ => panic!("IPv6 not supported"),
        }
    }
}

impl FromStr for SocketAddr {
    type Err = ();

    fn from_str(s: &str) -> Result<SocketAddr, ()> {
        let mut parts = s.split(':');
        let addr = parts.next().ok_or(())?.parse::<IpAddr>()?;
        let port = parts.next().ok_or(())?.parse::<u16>().map_err(|_| ())?;
        Ok(SocketAddr { addr, port })
    }
}

impl SocketAddr {
    /// Create a new SocketAddr from an IpAddr and a port
    pub fn new(addr: IpAddr, port: u16) -> SocketAddr {
        SocketAddr { addr, port }
    }
}

impl From<(IpAddr, u16)> for SocketAddr {
    fn from((addr, port): (IpAddr, u16)) -> SocketAddr {
        SocketAddr { addr, port }
    }
}

impl From<(Ipv4Addr, u16)> for SocketAddr {
    fn from((addr, port): (Ipv4Addr, u16)) -> SocketAddr {
        SocketAddr {
            addr: IpAddr::Ipv4(addr),
            port,
        }
    }
}

impl From<(Ipv6Addr, u16)> for SocketAddr {
    fn from((addr, port): (Ipv6Addr, u16)) -> SocketAddr {
        SocketAddr {
            addr: IpAddr::Ipv6(addr),
            port,
        }
    }
}

impl fmt::Display for SocketAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.addr, self.port)
    }
}

/// Convert a mask to a prefix length
pub fn mask_to_prefix(mask: IpAddr) -> Result<u8, Error> {
    match mask {
        IpAddr::Ipv4(Ipv4Addr(mask)) => {
            let mut mask = Ipv4Addr(mask).to_u32();
            let mut prefix = 0;
            while mask & (1 << 31) != 0 {
                prefix += 1;
                mask <<= 1;
            }
            if mask != 0 {
                Err(Error)
            } else {
                Ok(prefix)
            }
        }
        _ => panic!("IPv6 not supported"),
    }
}
