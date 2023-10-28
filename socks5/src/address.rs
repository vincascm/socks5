use std::{
    convert::{TryFrom, TryInto},
    fmt::{Debug, Display},
    future::Future,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
};

use bytes::{BufMut, Bytes, BytesMut};
use futures_lite::AsyncReadExt;
use tinyvec::ArrayVec;

use crate::{
    error::Error,
    message::Replies,
    ser::{Decode, Encode},
};

/// SOCKS5 address type
#[derive(Clone, Debug, PartialEq)]
pub enum Address {
    /// Socket address
    Socket(SocketAddr),
    /// Domain name address
    DomainName(Box<ArrayVec<[u8; 2048]>>, u16),
}

impl Address {
    pub async fn lookup<'a, F, T, E>(&'a self, f: F) -> Result<SocketAddr, Error>
    where
        F: Fn(&'a [u8], u16) -> T,
        T: Future<Output = Result<SocketAddr, E>>,
        E: Display,
    {
        let addr = match self {
            Address::Socket(addr) => *addr,
            Address::DomainName(name, port) => f(name, *port).await.map_err(|e| {
                Error::new(
                    Replies::HostUnreachable,
                    format!("domain \"{name}\" resolving failed: {e}"),
                )
            })?,
        };
        Ok(addr)
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for Address {
    async fn decode(r: &mut T) -> crate::error::Result<Self> {
        let addr_type = Self::read_u8(r).await?;
        let addr_type = AddressType::try_from(addr_type)?;

        match addr_type {
            AddressType::Ipv4 => {
                let mut buf = vec![0; 6];
                r.read_exact(&mut buf).await?;
                let v4addr: [u8; 4] = buf[1..=4].try_into()?;
                let v4addr: Ipv4Addr = v4addr.into();
                let port: [u8; 2] = buf[5..=6].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::Socket(SocketAddr::V4(SocketAddrV4::new(
                    v4addr, port,
                ))))
            }
            AddressType::Ipv6 => {
                let mut buf = vec![0; 18];
                r.read_exact(&mut buf).await?;
                let v6addr: [u8; 16] = buf[1..=16].try_into()?;
                let v6addr: Ipv6Addr = v6addr.into();
                let port: [u8; 2] = buf[17..=18].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::Socket(SocketAddr::V6(SocketAddrV6::new(
                    v6addr, port, 0, 0,
                ))))
            }
            AddressType::DomainName => {
                let domain_len = Self::read_u8(r).await? as usize;
                let mut buf = vec![0; domain_len + 2];
                r.read_exact(&mut buf).await?;
                let mut domain = ArrayVec::new();
                domain.extend_from_slice(&buf[..domain_len]);
                let port: [u8; 2] = buf[domain_len..domain_len + 2].try_into()?;
                let port = u16::from_be_bytes(port);
                Ok(Address::DomainName(Box::new(domain), port))
            }
        }
    }
}

impl Encode for Address {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
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
        Address::DomainName(Box::new(domain), port)
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
