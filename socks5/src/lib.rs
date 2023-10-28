//! Socks5 protocol definition (RFC1928)
//!
//! Implements [SOCKS Protocol Version 5](https://www.ietf.org/rfc/rfc1928) proxy protocol

pub mod address;
pub mod error;
pub mod head;
pub mod message;
pub mod ser;

const VERSION: u8 = 0x05;
