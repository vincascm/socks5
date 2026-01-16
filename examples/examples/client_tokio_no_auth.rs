use socks5_client::{connect, Error};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_util::compat::TokioAsyncReadCompatExt;

#[tokio::main]
async fn main() -> std::result::Result<(), Error> {
    let mut stream = TcpStream::connect("127.0.0.1:1080").await?.compat();

    let dest = ("google.com".as_bytes(), 80).into();
    connect(&mut stream, dest, None).await?;

    let mut stream = stream.into_inner();

    stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await?;
    stream.flush().await?;

    let mut buf = vec![];
    stream.read_to_end(&mut buf).await?;
    println!("{}", String::from_utf8_lossy(&buf));
    Ok(())
}
