//! Socks5 protocol definition (RFC1928)
//!
//! Implements [SOCKS Protocol Version 5](https://www.ietf.org/rfc/rfc1928) proxy protocol
//! some reference from
//! <https://github.com/shadowsocks/shadowsocks-rust/blob/master/src/relay/socks5.rs>

use std::net::SocketAddr;

use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

mod address;
mod error;
mod head;
mod message;

pub use address::Address;
pub use error::Error;
pub use head::{
    AuthenticationRequest, AuthenticationResponse, TcpRequestHeader, TcpResponseHeader,
};
pub use message::{Command, Method, Replies};

const VERSION: u8 = 0x05;

pub async fn connect_without_auth<A: ToSocketAddrs>(
    socks5_server_addr: A,
    dest_addr: SocketAddr,
) -> io::Result<TcpStream> {
    let mut srv = TcpStream::connect(socks5_server_addr).await?;
    // authentication
    let auth_req: AuthenticationRequest = vec![Method::NONE; 1].into();
    srv.write(&auth_req.to_bytes()).await?;
    let auth_resp = AuthenticationResponse::read_from(&mut srv).await?;
    if auth_resp.required_authentication() {
        return Err(Error::new(
            Replies::GeneralFailure,
            "server does not support none password auth method",
        )
        .into());
    }

    // requests
    let tcp_req = TcpRequestHeader::new(Command::Connect, dest_addr.into());
    srv.write(&tcp_req.to_bytes()).await?;
    let tcp_resp = TcpResponseHeader::read_from(&mut srv).await?;
    if tcp_resp.is_success() {
        Ok(srv)
    } else {
        Err(Error::new(
            Replies::GeneralFailure,
            "connection to socks5 server failed",
        )
        .into())
    }
}
