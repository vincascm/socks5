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
- **Multiple Async Runtimes**: Supports both `tokio` and `smol` runtimes via `futures-lite`.

## Usage

### Add as a Dependency

Add the corresponding crate to your `Cargo.toml` file:

```toml
[dependencies]
# If you need the SOCKS5 core protocol
socks5 = { git = "https://github.com/vincascm/socks5.git" }

# If you need client functionality
socks5-client = { git = "https://github.com/vincascm/socks5.git" }

# If you need server functionality
socks5-server = { git = "https://github.com/vincascm/socks5.git" }
```

### Examples

For client examples, see [examples](/examples/examples).
For the server, see [socks5-server](https://github.com/vincascm/socks5-server).

## Contributing

Contributions of any kind are welcome, whether it's submitting an issue or a pull request.

## License

This project is licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
