use std::{convert::TryFrom, fmt::Debug};

use bytes::{BufMut, Bytes, BytesMut};
use tokio::io::{self, AsyncRead, AsyncReadExt};

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
    pub async fn read_from<R>(r: &mut R) -> io::Result<AuthenticationRequest>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;
        if ver != VERSION {
            let err = Error::new(
                Replies::GeneralFailure,
                format_args!("unsupported socks version {:#x}", ver),
            );
            return Err(err.into());
        }

        let n = r.read_u8().await?;
        let mut methods = Vec::new();
        for _ in 0..(n as usize) {
            let method = r.read_u8().await?;
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
    pub async fn read_from<R>(r: &mut R) -> io::Result<AuthenticationResponse>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;
        if ver != VERSION {
            let err = Error::new(
                Replies::GeneralFailure,
                format_args!("unsupported socks version {:#x}", ver),
            );
            return Err(err.into());
        }

        let method = r.read_u8().await?;
        let method = Method::try_from(method)?;
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

    pub async fn read_from<R>(r: &mut R) -> Result<TcpRequestHeader, Error>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;
        if ver != VERSION {
            return Err(Error::new(
                Replies::ConnectionRefused,
                format_args!("unsupported socks version {:#x}", ver),
            ));
        }

        let command = r.read_u8().await?;
        let command = Command::try_from(command)?;
        // skip RSV field
        r.read_u8().await?;

        let address = Address::read_from(r).await?;
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
    reply: Replies,
    /// Reply address
    address: Address,
}

impl TcpResponseHeader {
    /// Creates a response header
    pub fn new(reply: Replies, address: Address) -> TcpResponseHeader {
        TcpResponseHeader { reply, address }
    }

    pub async fn read_from<R>(r: &mut R) -> Result<TcpResponseHeader, Error>
    where
        R: AsyncRead + Unpin,
    {
        let ver = r.read_u8().await?;
        if ver != VERSION {
            return Err(Error::new(
                Replies::ConnectionRefused,
                format_args!("unsupported socks version {:#x}", ver),
            ));
        }

        let reply = r.read_u8().await?;
        let reply = Replies::try_from(reply)?;
        // skip RSV field
        r.read_u8().await?;

        let address = Address::read_from(r).await?;
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
