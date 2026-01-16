use futures_lite::{AsyncReadExt, AsyncWriteExt};
use smol::net::TcpStream;
use socks5_client::{connect, Error};

fn main() -> Result<(), Error> {
    smol::block_on(async {
        let mut stream = TcpStream::connect("127.0.0.1:1080").await?;

        connect(&mut stream, ("google.com".as_bytes(), 80).into(), None).await?;

        stream.write_all(b"GET / HTTP/1.0\r\n\r\n").await?;
        stream.flush().await?;

        let mut buf = vec![];
        stream.read_to_end(&mut buf).await?;
        println!("{}", String::from_utf8_lossy(&buf));
        Ok(())
    })
}
