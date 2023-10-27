use std::{
    convert::{TryFrom, TryInto},
    fmt::{Debug, Display},
    future::Future,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use bytes::{BufMut, Bytes, BytesMut};
use tinyvec::ArrayVec;

use crate::{Error, Replies};

/// SOCKS5 address type
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    /// Socket address
    Socket(SocketAddr),
    /// Domain name address
    DomainName(ArrayVec<[u8; 2048]>, u16),
}

impl Address {
    pub fn from_bytes(buf: &[u8]) -> Result<Address, Error> {
        let addr_type = AddressType::try_from(buf[0])?;

        match addr_type {
            AddressType::Ipv4 => {
                let v4addr: [u8; 4] = buf[1..=4].try_into()?;
                let v4addr: Ipv4Addr = v4addr.into();
                let port: [u8; 2] = buf[5..=6].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::Socket(SocketAddr::V4(SocketAddrV4::new(
                    v4addr, port,
                ))))
            }
            AddressType::Ipv6 => {
                let v6addr: [u8; 16] = buf[1..=16].try_into()?;
                let v6addr: Ipv6Addr = v6addr.into();
                let port: [u8; 2] = buf[17..=18].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::Socket(SocketAddr::V6(SocketAddrV6::new(
                    v6addr, port, 0, 0,
                ))))
            }
            AddressType::DomainName => {
                let domain_len = buf[1] as usize;
                let domain_end = 2 + domain_len;
                let mut domain = ArrayVec::new();
                domain.extend_from_slice(&buf[2..domain_end]);
                let port: [u8; 2] = buf[domain_end..domain_end + 2].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::DomainName(domain, port))
            }
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::with_capacity(self.size_hint());
        match self {
            Address::Socket(addr) => match addr {
                SocketAddr::V4(addr) => {
                    buffer.put_u8(AddressType::Ipv4 as u8);
                    buffer.put_slice(&addr.ip().octets());
                    buffer.put_u16(addr.port());
                }
                SocketAddr::V6(addr) => {
                    buffer.put_u8(AddressType::Ipv6 as u8);
                    for seg in &addr.ip().segments() {
                        buffer.put_u16(*seg);
                    }
                    buffer.put_u16(addr.port());
                }
            },
            Address::DomainName(dnaddr, port) => {
                buffer.put_u8(AddressType::DomainName as u8);
                buffer.put_u8(dnaddr.len() as u8);
                buffer.put_slice(dnaddr);
                buffer.put_u16(*port);
            }
        }
        buffer.freeze()
    }

    pub(crate) fn size_hint(&self) -> usize {
        let type_length = match &self {
            // addr len + port len
            Address::Socket(SocketAddr::V4(..)) => 4 + 2,
            // addr len + port len
            Address::Socket(SocketAddr::V6(..)) => 8 * 2 + 2,
            // domain len + domain self len + port len
            Address::DomainName(d, _) => 1 + d.len() + 2,
        };
        // add 1 version byte length
        1 + type_length
    }

    pub async fn lookup<'a, F, T, E>(&'a self, f: F) -> Result<SocketAddr, Error>
    where
        F: Fn(&'a [u8]) -> T,
        T: Future<Output = Result<IpAddr, E>>,
        E: Display,
    {
        let addr = match self {
            Address::Socket(addr) => *addr,
            Address::DomainName(name, port) => {
                let addr = f(name).await.map_err(|e| {
                    Error::new(
                        Replies::HostUnreachable,
                        format!("domain \"{name}\" resolving failed: {e}"),
                    )
                })?;
                (addr, *port).into()
            }
        };
        Ok(addr)
    }
}

impl From<SocketAddr> for Address {
    fn from(s: SocketAddr) -> Address {
        Address::Socket(s)
    }
}

impl From<SocketAddrV4> for Address {
    fn from(s: SocketAddrV4) -> Address {
        let s: SocketAddr = s.into();
        Address::Socket(s)
    }
}

impl From<SocketAddrV6> for Address {
    fn from(s: SocketAddrV6) -> Address {
        let s: SocketAddr = s.into();
        Address::Socket(s)
    }
}

impl<'a, T: Into<&'a [u8]>> From<(T, u16)> for Address {
    fn from((host, port): (T, u16)) -> Address {
        let s = host.into();
        let mut domain = ArrayVec::new();
        domain.extend_from_slice(s);
        Address::DomainName(domain, port)
    }
}

#[derive(Copy, Clone)]
enum AddressType {
    Ipv4 = 1,
    DomainName = 3,
    Ipv6 = 4,
}

impl TryFrom<u8> for AddressType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let m = match value {
            1 => AddressType::Ipv4,
            3 => AddressType::DomainName,
            4 => AddressType::Ipv6,
            c => {
                return Err(Error::new(
                    Replies::AddressTypeNotSupported,
                    format!("unsupported address type {:#x}", c),
                ))
            }
        };
        Ok(m)
    }
}
