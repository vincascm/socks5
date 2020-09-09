//! Socks5 protocol definition (RFC1928)
//!
//! Implements [SOCKS Protocol Version 5](https://www.ietf.org/rfc/rfc1928) proxy protocol

mod address;
mod error;
mod funcs;
mod head;
mod message;

pub use address::Address;
pub use error::Error;
pub use funcs::connect_without_auth;
pub use head::{
    AuthenticationRequest, AuthenticationResponse, TcpRequestHeader, TcpResponseHeader,
};
pub use message::{Command, Method, Replies};

const VERSION: u8 = 0x05;
