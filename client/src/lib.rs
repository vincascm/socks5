use futures_lite::{AsyncReadExt, AsyncWriteExt};
use socks5::{
    address::Address,
    error::Error as SocksError,
    head::{
        AuthenticationRequest, AuthenticationResponse, SubNegotiationAuthenticationRequest,
        SubNegotiationAuthenticationResponse, TcpRequestHeader, TcpResponseHeader,
    },
    message::{Command, Method},
    ser::{Decode, Encode},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("socks error: {0}")]
    Socks(#[from] SocksError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("unsupport method GssApi")]
    UnsupportMethodGssApi,
    #[error("require auth")]
    RequireAuth,
    #[error("not acceptable")]
    NotAcceptable,
    #[error("auth failure")]
    AuthFailure,
}

pub async fn connect<T>(
    connect: &mut T,
    dest: Address,
    auth_info: Option<SubNegotiationAuthenticationRequest>,
) -> Result<()>
where
    T: AsyncReadExt + AsyncWriteExt + Unpin,
{
    let method = match auth_info {
        Some(_) => Method::PASSWORD,
        None => Method::NONE,
    };
    let auth_req: AuthenticationRequest = [method; 1].as_slice().into();
    write(auth_req, connect).await?;
    let auth_resp = AuthenticationResponse::read(connect).await?;
    match auth_resp.method {
        Method::NONE => {}
        Method::PASSWORD => match auth_info {
            Some(a) => {
                connect.write_all(&a.encode()).await?;
                connect.flush().await?;
                let auth_resp = SubNegotiationAuthenticationResponse::decode(connect).await?;
                if !auth_resp.success() {
                    return Err(Error::AuthFailure);
                }
            }
            None => return Err(Error::RequireAuth),
        },
        Method::NotAcceptable => return Err(Error::NotAcceptable),
        Method::GSSAPI => return Err(Error::UnsupportMethodGssApi),
    }

    let tcp_req = TcpRequestHeader::new(Command::Connect, dest);
    write(tcp_req, connect).await?;
    let tcp_resp = TcpResponseHeader::read(connect).await?;
    if tcp_resp.is_success() {
        Ok(())
    } else {
        let e: SocksError = tcp_resp.reply.into();
        Err(e.into())
    }
}

async fn write<T: Encode, C: AsyncWriteExt + Unpin>(head: T, c: &mut C) -> Result<()> {
    c.write_all(&head.as_bytes()).await?;
    c.flush().await?;
    Ok(())
}
