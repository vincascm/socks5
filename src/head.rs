use std::{convert::TryFrom, fmt::Debug, net::TcpStream};

use bytes::{BufMut, Bytes, BytesMut};
use futures::io::AsyncReadExt;
use smol::Async;

use crate::{Address, Command, Error, Method, Replies, VERSION};

/// SOCKS5 authentication request packet
///
/// ```plain
/// +----+----------+----------+
/// |VER | NMETHODS | METHODS  |
/// +----+----------+----------+
/// | 5  |    1     | 1 to 255 |
/// +----+----------+----------|
/// ```
pub struct AuthenticationRequest {
    methods: Vec<Method>,
}

impl AuthenticationRequest {
    pub async fn read_from(stream: &mut Async<TcpStream>) -> Result<AuthenticationRequest, Error> {
        let mut buf = [0; 257];
        stream.read(&mut buf).await?;
        check_version(buf[0])?;
        let n = buf[1] as usize;
        let mut methods = Vec::new();
        for i in 0..n {
            let method = buf[2 + i];
            let method = Method::try_from(method)?;
            methods.push(method);
        }
        Ok(AuthenticationRequest { methods })
    }

    pub fn required_authentication(&self) -> bool {
        !self.methods.contains(&Method::NONE)
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(VERSION);
        buffer.put_u8(self.methods.len() as u8);
        for i in &self.methods {
            buffer.put_u8(*i as u8);
        }
        buffer.freeze()
    }
}

impl From<Vec<Method>> for AuthenticationRequest {
    fn from(methods: Vec<Method>) -> Self {
        Self { methods }
    }
}

/// SOCKS5 authentication response packet
///
/// ```plain
/// +----+--------+
/// |VER | METHOD |
/// +----+--------+
/// | 1  |   1    |
/// +----+--------+
/// ```
pub struct AuthenticationResponse {
    method: Method,
}

impl AuthenticationResponse {
    pub async fn read_from(stream: &mut Async<TcpStream>) -> Result<AuthenticationResponse, Error> {
        let mut buf = [0; 2];
        stream.read(&mut buf).await?;
        check_version(buf[0])?;
        let method = Method::try_from(buf[1])?;
        Ok(AuthenticationResponse { method })
    }

    pub fn required_authentication(&self) -> bool {
        self.method != Method::NONE
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::new();
        buffer.put_u8(VERSION);
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

    pub async fn read_from(stream: &mut Async<TcpStream>) -> Result<TcpRequestHeader, Error> {
        let mut buf = [0; 259];
        stream.read(&mut buf).await?;
        check_version(buf[0])?;
        let command = Command::try_from(buf[1])?;
        let address = Address::from_bytes(&buf[3..])?;
        Ok(TcpRequestHeader { command, address })
    }

    pub fn address(&self) -> &Address {
        &self.address
    }

    pub fn command(&self) -> Command {
        self.command
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::with_capacity(3 + self.address.len());
        buffer.put_u8(VERSION);
        buffer.put_u8(self.command as u8);
        buffer.put_u8(0);
        buffer.put_slice(&self.address.to_bytes());
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

    pub async fn read_from(stream: &mut Async<TcpStream>) -> Result<TcpResponseHeader, Error> {
        let mut buf = [0; 259];
        stream.read(&mut buf).await?;
        check_version(buf[0])?;
        let reply = Replies::try_from(buf[1])?;
        let address = Address::from_bytes(&buf[3..])?;
        Ok(TcpResponseHeader { reply, address })
    }

    pub fn is_success(&self) -> bool {
        self.reply == Replies::Succeeded
    }

    pub fn to_bytes(&self) -> Bytes {
        let mut buffer = BytesMut::with_capacity(3 + self.address.len());
        buffer.put_u8(VERSION);
        buffer.put_u8(self.reply as u8);
        buffer.put_u8(0);
        buffer.put_slice(&self.address.to_bytes());
        buffer.freeze()
    }
}

fn check_version(v: u8) -> Result<(), Error> {
    if v == VERSION {
        Ok(())
    } else {
        Err(Error::new(
            Replies::ConnectionRefused,
            format!("unsupported socks version {:#x}", v),
        ))
    }
}
