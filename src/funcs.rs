use futures_util::{AsyncRead, AsyncWrite, AsyncWriteExt};

use crate::{
    Address, AuthenticationRequest, AuthenticationResponse, Command, Error, Method, Replies,
    TcpRequestHeader, TcpResponseHeader,
};

pub async fn connect_without_auth<T: AsyncRead + AsyncWrite + Unpin>(
    socks5_server_connect: &mut T,
    dest_addr: Address,
) -> Result<(), Error> {
    let mut c = socks5_server_connect;
    // authentication
    let auth_req: AuthenticationRequest = vec![Method::NONE; 1].into();
    c.write_all(&auth_req.to_bytes()).await?;
    c.flush().await?;
    let auth_resp = AuthenticationResponse::read_from(&mut c).await?;
    if auth_resp.required_authentication() {
        return Err(Error::new(
            Replies::GeneralFailure,
            "server does not support none password auth method",
        ));
    }

    // requests
    let tcp_req = TcpRequestHeader::new(Command::Connect, dest_addr);
    c.write_all(&tcp_req.to_bytes()).await?;
    c.flush().await?;
    let tcp_resp = TcpResponseHeader::read_from(&mut c).await?;
    if tcp_resp.is_success() {
        Ok(())
    } else {
        Err(tcp_resp.reply.into())
    }
}
