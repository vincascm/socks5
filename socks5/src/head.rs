use core::convert::TryFrom;

use bytes::{BufMut, Bytes, BytesMut};
use futures_lite::AsyncReadExt;
use tinyvec::ArrayVec;

use crate::{
    address::Address,
    error::Result,
    message::{Command, Method, Replies},
    ser::{Decode, Encode},
};

/// SOCKS5 authentication request packet
#[derive(Debug)]
pub struct AuthenticationRequest {
    methods: ArrayVec<[Method; 256]>,
}

impl AuthenticationRequest {
    pub fn required_authentication(&self) -> bool {
        !self.methods.contains(&Method::NONE)
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for AuthenticationRequest {
    async fn decode(r: &mut T) -> Result<Self> {
        let n = Self::read_u8(r).await? as usize;
        let mut buf = vec![0; n * Method::size_hint()];
        r.read_exact(&mut buf).await?;
        let mut methods = ArrayVec::new();
        for i in buf {
            let method = Method::try_from(i)?;
            methods.push(method);
        }
        Ok(AuthenticationRequest { methods })
    }
}

impl Encode for AuthenticationRequest {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.methods.len() as u8);
        for i in &self.methods {
            buffer.put_u8(*i as u8);
        }
        buffer.freeze()
    }
}

impl<'a> From<&'a [Method]> for AuthenticationRequest {
    fn from(m: &'a [Method]) -> Self {
        let mut methods = ArrayVec::new();
        methods.extend_from_slice(m);
        Self { methods }
    }
}

/// SOCKS5 authentication response packet
pub struct AuthenticationResponse {
    method: Method,
}

impl AuthenticationResponse {
    pub fn required_authentication(&self) -> bool {
        self.method != Method::NONE
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for AuthenticationResponse {
    async fn decode(r: &mut T) -> Result<Self> {
        let method = Self::read_u8(r).await?;
        let method = Method::try_from(method)?;
        Ok(AuthenticationResponse { method })
    }
}

impl Encode for AuthenticationResponse {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.method as u8);
        buffer.freeze()
    }
}

impl From<Method> for AuthenticationResponse {
    fn from(method: Method) -> AuthenticationResponse {
        AuthenticationResponse { method }
    }
}

/// TCP request header after authentication
///
/// ```plain
/// +----+-----+-------+------+----------+----------+
/// |VER | CMD |  RSV  | ATYP | DST.ADDR | DST.PORT |
/// +----+-----+-------+------+----------+----------+
/// | 1  |  1  | X'00' |  1   | Variable |    2     |
/// +----+-----+-------+------+----------+----------+
/// ```
pub struct TcpRequestHeader {
    /// SOCKS5 command
    command: Command,
    /// Remote address
    address: Address,
}

impl TcpRequestHeader {
    pub fn new(command: Command, address: Address) -> Self {
        Self { command, address }
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn command(&self) -> Command {
        self.command
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for TcpRequestHeader {
    async fn decode(r: &mut T) -> Result<Self> {
        let command = Self::read_u8(r).await?;
        let command = Command::try_from(command)?;
        Self::read_u8(r).await?;
        let address = Address::decode(r).await?;
        Ok(TcpRequestHeader { command, address })
    }
}

impl Encode for TcpRequestHeader {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.command as u8);
        buffer.put_u8(0);
        buffer.put_slice(&self.address.encode());
        buffer.freeze()
    }
}

/// TCP response header
///
/// ```plain
/// +----+-----+-------+------+----------+----------+
/// |VER | REP |  RSV  | ATYP | BND.ADDR | BND.PORT |
/// +----+-----+-------+------+----------+----------+
/// | 1  |  1  | X'00' |  1   | Variable |    2     |
/// +----+-----+-------+------+----------+----------+
/// ```
#[derive(Debug)]
pub struct TcpResponseHeader {
    /// SOCKS5 reply
    pub reply: Replies,
    /// Reply address
    address: Address,
}

impl TcpResponseHeader {
    /// Creates a response header
    pub fn new(reply: Replies, address: Address) -> TcpResponseHeader {
        TcpResponseHeader { reply, address }
    }

    pub fn is_success(&self) -> bool {
        self.reply == Replies::Succeeded
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for TcpResponseHeader {
    async fn decode(r: &mut T) -> Result<Self> {
        let reply = Self::read_u8(r).await?;
        let reply = Replies::try_from(reply)?;
        Self::read_u8(r).await?;
        let address = Address::decode(r).await?;
        Ok(TcpResponseHeader { reply, address })
    }
}

impl Encode for TcpResponseHeader {
    fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(self.reply as u8);
        buffer.put_u8(0);
        buffer.put_slice(&self.address.encode());
        buffer.freeze()
    }
}
