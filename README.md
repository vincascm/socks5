# SOCKS5 Protocol Implementation

[中文](/README_zh.md)

This is an `async` SOCKS5 protocol (RFC 1928) implementation library written in Rust. This project is a Cargo workspace containing the following core parts.

## Project Structure

- `socks5`: The core library that defines all the necessary data structures and serialization/deserialization logic for the SOCKS5 protocol, such as address types, request/response messages, authentication methods, etc.
- `socks5-client`: A SOCKS5 client library. It provides a `connect` function used to connect to a destination address through a SOCKS5 proxy server.
- `socks5-server`: A SOCKS5 server library. It provides a `proxy` function to handle incoming SOCKS5 connections, currently supporting the `CONNECT` command.

## Features

- **Fully Async**: The entire library is built on Rust's `async/await` model, making it suitable for high-performance network applications.
- **Modular Design**: Clearly separates the core protocol, client, and server logic using a Cargo workspace.
- **Multiple Authentication Methods**: The client supports `No Authentication` and `Username/Password` authentication. The server currently implements the `No Authentication` mode.
- **Multiple Address Types**: Supports IPv4, IPv6, and domain name address types.
- **RFC 1928 Compliant**: Strictly follows the SOCKS5 protocol standard.

## Usage

### Add as a Dependency

Add the corresponding crate to your `Cargo.toml` file:

```toml
[dependencies]
# If you need the SOCKS5 core protocol
socks5 = { git = "https://github.com/vincascm/socks5.git", branch = "main" }

# If you need client functionality
socks5-client = { git = "https://github.com/vincascm/socks5.git", branch = "main" }

# If you need server functionality
socks5-server = { git = "https://github.com/vincascm/socks5.git", branch = "main" }
```
*Please replace `your-repo` with your repository address.*

### Client Example

Here is an example of using `socks5-client` to connect to `example.com:80`.

```rust
use async_io::Async;
use futures_lite::io::Cursor;
use socks5::address::Address;
use socks5_client::{connect, Result};
use std::net::TcpStream;

async fn client_example() -> Result<()> {
    // Create an in-memory stream for demonstration
    let mut stream = Cursor::new(Vec::new());

    // Destination address
    let dest_addr = Address::from(( "example.com", 80));

    // Perform the connection, without authentication here
    connect(&mut stream, dest_addr, None).await?;

    println!("Successfully connected via SOCKS5 proxy!");
    Ok(())
}
```

### Server Example

Here is an example of using `socks5-server` to start a simple proxy server.

```rust
use async_io::Async;
use futures_lite::io::Cursor;
use socks5_server::{proxy, Result};
use std::net::{SocketAddr, Ipv4Addr};

async fn server_example() -> Result<()> {
    // Simulate a client connection
    let mut stream = Cursor::new(Vec::new());

    // Simulate the client address
    let src_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 8080);

    // Run the proxy logic
    // Note: In a real application, the stream should be a real TCP connection from the network
    proxy(&mut stream, src_addr).await?;

    println!("Proxy logic completed!");
    Ok(())
}
```

## Contributing

Contributions of any kind are welcome, whether it's submitting an issue or a pull request.

## License

This project is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
