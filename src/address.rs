use std::{
    convert::{TryFrom, TryInto},
    fmt::Debug,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, ToSocketAddrs},
};

use bytes::{BufMut, Bytes, BytesMut};

use crate::{Error, Replies};

/// SOCKS5 address type
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    /// Socket address (IP Address)
    SocketAddress(SocketAddr),
    /// Domain name address
    DomainNameAddress(String, u16),
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
                Ok(Address::SocketAddress(SocketAddr::V4(SocketAddrV4::new(
                    v4addr, port,
                ))))
            }
            AddressType::Ipv6 => {
                let v6addr: [u8; 16] = buf[1..=16].try_into()?;
                let v6addr: Ipv6Addr = v6addr.into();
                let port: [u8; 2] = buf[17..=18].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::SocketAddress(SocketAddr::V6(SocketAddrV6::new(
                    v6addr, port, 0, 0,
                ))))
            }
            AddressType::DomainName => {
                let domain_len = buf[1] as usize;
                let domain_end = 2 + domain_len;
                let domain = match String::from_utf8(buf[2..domain_end].to_vec()) {
                    Ok(domain) => domain,
                    Err(_) => {
                        return Err(Error::new(
                            Replies::GeneralFailure,
                            format_args!("invalid address encoding"),
                        ))
                    }
                };
                let port: [u8; 2] = buf[domain_end..domain_end + 2].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::DomainNameAddress(domain, port))
            }
        }
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::with_capacity(self.len());
        match self {
            Address::SocketAddress(addr) => match addr {
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
            Address::DomainNameAddress(dnaddr, port) => {
                buffer.put_u8(AddressType::DomainName as u8);
                buffer.put_u8(dnaddr.len() as u8);
                buffer.put_slice(dnaddr[..].as_bytes());
                buffer.put_u16(*port);
            }
        }
        buffer.freeze()
    }

    pub(crate) fn len(&self) -> usize {
        match self {
            // VER + addr len + port len
            Address::SocketAddress(SocketAddr::V4(..)) => 1 + 4 + 2,
            // VER + addr len + port len
            Address::SocketAddress(SocketAddr::V6(..)) => 1 + 8 * 2 + 2,
            // VER + domain len + domain self len + port len
            Address::DomainNameAddress(ref d, _) => 1 + 1 + d.len() + 2,
        }
    }

    pub async fn to_socket_addrs(self) -> Result<SocketAddr, Error> {
        Ok(match self {
            Address::SocketAddress(addr) => addr,
            Address::DomainNameAddress(addr, port) => {
                let mut addr = smol::blocking!((addr.as_str(), port).to_socket_addrs())?;
                addr.next().ok_or_else(||Error::new(
                    Replies::GeneralFailure,
                    format_args!("domain resolving failed"),
                ))?
            }
        })
    }
}

impl From<SocketAddr> for Address {
    fn from(s: SocketAddr) -> Address {
        Address::SocketAddress(s)
    }
}

impl From<SocketAddrV4> for Address {
    fn from(s: SocketAddrV4) -> Address {
        let s: SocketAddr = s.into();
        Address::SocketAddress(s)
    }
}

impl From<SocketAddrV6> for Address {
    fn from(s: SocketAddrV6) -> Address {
        let s: SocketAddr = s.into();
        Address::SocketAddress(s)
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
                    format_args!("unsupported address type {:#x}", c),
                ))
            }
        };
        Ok(m)
    }
}
