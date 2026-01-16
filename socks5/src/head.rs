use core::convert::TryFrom;

use bytes::{BufMut, Bytes, BytesMut};
use futures_lite::AsyncReadExt;
use tinyvec::ArrayVec;

use crate::{
    address::Address,
    error::{Error, Result},
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
    pub method: Method,
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

pub struct SubNegotiationAuthenticationRequest {
    username: String,
    password: String,
}

impl SubNegotiationAuthenticationRequest {
    pub fn new(username: &str, passowrd: &str) -> Result<Self> {
        if username.len() > 255 || passowrd.len() > 255 {
            Err(Error::new(
                Replies::GeneralFailure,
                "invalid length of username or password",
            ))
        } else {
            Ok(Self {
                username: username.to_string(),
                password: passowrd.to_string(),
            })
        }
    }

    pub fn encode(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(1);
        buffer.put_u8(self.username.len() as u8);
        buffer.put_slice(self.username.as_bytes());
        buffer.put_u8(self.password.len() as u8);
        buffer.put_slice(self.password.as_bytes());
        buffer.freeze()
    }
}

impl<T: AsRef<str>, U: AsRef<str>> From<(T, U)> for SubNegotiationAuthenticationRequest {
    fn from((username, password): (T, U)) -> Self {
        Self {
            username: username.as_ref().to_string(),
            password: password.as_ref().to_string(),
        }
    }
}

pub struct SubNegotiationAuthenticationResponse {
    result: u8,
}

impl SubNegotiationAuthenticationResponse {
    pub fn success(&self) -> bool {
        self.result == 0
    }
}

impl<T: AsyncReadExt + Unpin> Decode<T> for SubNegotiationAuthenticationResponse {
    async fn decode(r: &mut T) -> Result<Self> {
        let version = Self::read_u8(r).await?;
        if version != 1 {
            return Err(Error::new(
                Replies::GeneralFailure,
                "invalid sub negotiation version",
            ));
        }
        let result = Self::read_u8(r).await?;
        Ok(SubNegotiationAuthenticationResponse { result })
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
