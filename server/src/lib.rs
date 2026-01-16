use std::{
    io::ErrorKind,
    net::{SocketAddr, TcpStream},
};

use async_io::Async;
use futures_lite::{
    AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt,
    future::race,
    io::{copy, split},
};
use socks5::{
    error::Error as SocksError,
    head::{AuthenticationRequest, AuthenticationResponse, TcpRequestHeader},
    message::{Command, Method, Replies},
    ser::{Decode, Encode},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("socks error: {0}")]
    Socks(#[from] SocksError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn proxy<'a, T: AsyncReadExt + AsyncWriteExt + Unpin + 'a>(
    connect: &mut T,
    src: SocketAddr,
) -> Result<()>
where
    &'a T: AsyncRead + AsyncWrite,
{
    // authentication
    let authentication_request = AuthenticationRequest::read(connect).await?;
    let authentication_response: AuthenticationResponse =
        if authentication_request.required_authentication() {
            Method::NotAcceptable
        } else {
            Method::NONE
        }
        .into();
    write(authentication_response, connect).await?;

    // requests
    let header = match TcpRequestHeader::read(connect).await {
        Ok(v) => v,
        Err(e) => {
            let resp = e.reply.into_response(src.into());
            write(resp, connect).await?;
            return Err(e.into());
        }
    };
    let addr = header.address();
    match header.command() {
        Command::Connect => {
            let dest_addr = match addr.lookup(lookup).await {
                Ok(addr) => addr,
                Err(e) => {
                    let resp = e.reply.into_response(addr.clone());
                    write(resp, connect).await?;
                    return Err(e.into());
                }
            };
            let dest_tcp = match Async::<TcpStream>::connect(dest_addr).await {
                Ok(s) => {
                    reply(Replies::Succeeded, dest_addr, connect).await?;
                    s
                }
                Err(e) => return Err(e.into()),
            };

            let (r, w) = split(connect);
            Ok(race(copy(r, &dest_tcp), copy(&dest_tcp, w))
                .await
                .map(|_| ())?)
        }
        // Bind and UdpAssociate, is not supported
        _ => {
            let rh = Replies::CommandNotSupported.into_response(addr.clone());
            write(rh, connect).await
        }
    }
}

async fn write<T: Encode, C: AsyncWriteExt + Unpin>(head: T, c: &mut C) -> Result<()> {
    c.write_all(&head.as_bytes()).await?;
    c.flush().await?;
    Ok(())
}

async fn reply<C: AsyncWriteExt + Unpin>(
    reply: Replies,
    addr: SocketAddr,
    c: &mut C,
) -> Result<()> {
    let header = reply.into_response(addr.into());
    write(header, c).await
}

async fn lookup(name: &[u8], port: u16) -> std::io::Result<SocketAddr> {
    let name = String::from_utf8_lossy(name);
    let addrs = async_dns::lookup(&name).await?;
    let addr = match addrs.into_iter().next() {
        Some(addr) => addr.ip_address,
        None => return Err(ErrorKind::AddrNotAvailable.into()),
    };
    Ok((addr, port).into())
}
