use std::{
    convert::TryFrom,
    fmt::Debug,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use bytes::{BufMut, Bytes, BytesMut};
use tokio::{
    io::{self, AsyncRead, AsyncReadExt},
    net::lookup_host,
};

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
    pub async fn read_from<R>(stream: &mut R) -> Result<Address, Error>
    where
        R: AsyncRead + Unpin,
    {
        let addr_type = stream.read_u8().await?;
        let addr_type = AddressType::try_from(addr_type)?;
        match addr_type {
            AddressType::Ipv4 => {
                let v4addr: Ipv4Addr = stream.read_u32().await?.into();
                let port = stream.read_u16().await?;
                Ok(Address::SocketAddress(SocketAddr::V4(SocketAddrV4::new(
                    v4addr, port,
                ))))
            }
            AddressType::Ipv6 => {
                let v6addr: Ipv6Addr = stream.read_u128().await?.into();
                let port = stream.read_u16().await?;
                Ok(Address::SocketAddress(SocketAddr::V6(SocketAddrV6::new(
                    v6addr, port, 0, 0,
                ))))
            }
            AddressType::DomainName => {
                let domain_len = stream.read_u8().await? as usize;
                let mut domain = vec![0; domain_len];
                stream.read_exact(&mut domain).await?;
                let domain = match String::from_utf8(domain.to_vec()) {
                    Ok(domain) => domain,
                    Err(_) => {
                        return Err(Error::new(
                            Replies::GeneralFailure,
                            "invalid address encoding",
                        ))
                    }
                };
                let port = stream.read_u16().await?;
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

    pub async fn to_socket_addrs(&self) -> io::Result<SocketAddr> {
        match self {
            Address::SocketAddress(addr) => Ok(*addr),
            Address::DomainNameAddress(addr, port) => {
                match lookup_host((addr.as_str(), *port)).await?.next() {
                    Some(addr) => Ok(addr),
                    None => Err(io::ErrorKind::AddrNotAvailable.into()),
                }
            }
        }
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
                    Replies::GeneralFailure,
                    format_args!("unsupported address type {:#x}", c),
                ))
            }
        };
        Ok(m)
    }
}
