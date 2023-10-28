use anyhow::{bail, Result};
use futures_lite::{AsyncReadExt, AsyncWriteExt};
use socks5::{
    address::Address,
    error::Error,
    head::{AuthenticationRequest, AuthenticationResponse, TcpRequestHeader, TcpResponseHeader},
    message::{Command, Method},
    ser::{Decode, Encode},
};

pub async fn connect_without_auth<T>(connect: &mut T, dest: Address) -> Result<()>
where
    T: AsyncReadExt + AsyncWriteExt + Unpin,
{
    // authentication
    let auth_req: AuthenticationRequest = [Method::NONE; 1].as_slice().into();
    write(auth_req, connect).await?;
    let auth_resp = AuthenticationResponse::read(connect).await?;
    if auth_resp.required_authentication() {
        bail!("server does not support none password auth method");
    }

    // requests
    let tcp_req = TcpRequestHeader::new(Command::Connect, dest);
    write(tcp_req, connect).await?;
    let tcp_resp = TcpResponseHeader::read(connect).await?;
    if tcp_resp.is_success() {
        Ok(())
    } else {
        let e: Error = tcp_resp.reply.into();
        bail!("connect failure: {e}");
    }
}

async fn write<T: Encode, C: AsyncWriteExt + Unpin>(head: T, c: &mut C) -> Result<()> {
    c.write_all(&head.as_bytes()).await?;
    c.flush().await?;
    Ok(())
}
