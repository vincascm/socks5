use std::net::SocketAddr;

use tokio::{
    io::{self, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

use crate::{
    AuthenticationRequest, AuthenticationResponse, Command, Error, Method, Replies,
    TcpRequestHeader, TcpResponseHeader,
};

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
