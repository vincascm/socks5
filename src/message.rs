use core::{convert::TryFrom, fmt::Debug};

use crate::{Address, Error, TcpResponseHeader};

#[derive(Default, Copy, Clone, PartialEq)]
pub enum Method {
    #[default]
    NONE = 0x00,
    GSSAPI = 0x01,
    PASSWORD = 0x02,
    NotAcceptable = 0xff,
}

impl TryFrom<u8> for Method {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let m = match value {
            0x00 => Method::NONE,
            0x01 => Method::GSSAPI,
            0x02 => Method::PASSWORD,
            0xff => Method::NotAcceptable,
            c => {
                return Err(Error::new(
                    Replies::GeneralFailure,
                    format_args!("unsupported method {:#x}", c),
                ))
            }
        };
        Ok(m)
    }
}

#[derive(Copy, Clone)]
pub enum Command {
    Connect = 0x01,
    Bind = 0x02,
    UdpAssociate = 0x03,
}

impl TryFrom<u8> for Command {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let c = match value {
            0x01 => Command::Connect,
            0x02 => Command::Bind,
            0x03 => Command::UdpAssociate,
            c => {
                return Err(Error::new(
                    Replies::GeneralFailure,
                    format_args!("unsupported command {:#x}", c),
                ))
            }
        };
        Ok(c)
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Replies {
    Succeeded = 0x00,
    GeneralFailure = 0x01,
    ConnectionNotAllowed = 0x02,
    NetworkUnreachable = 0x03,
    HostUnreachable = 0x04,
    ConnectionRefused = 0x05,
    TtlExpired = 0x06,
    CommandNotSupported = 0x07,
    AddressTypeNotSupported = 0x08,
}

impl Replies {
    pub fn into_response(self, address: Address) -> TcpResponseHeader {
        TcpResponseHeader::new(self, address)
    }
}

impl TryFrom<u8> for Replies {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let r = match value {
            0x00 => Replies::Succeeded,
            0x01 => Replies::GeneralFailure,
            0x02 => Replies::ConnectionNotAllowed,
            0x03 => Replies::NetworkUnreachable,
            0x04 => Replies::HostUnreachable,
            0x05 => Replies::ConnectionRefused,
            0x06 => Replies::TtlExpired,
            0x07 => Replies::CommandNotSupported,
            0x08 => Replies::AddressTypeNotSupported,
            c => {
                return Err(Error::new(
                    Replies::GeneralFailure,
                    format_args!("unsupported reply {:#x}", c),
                ))
            }
        };
        Ok(r)
    }
}

impl From<std::io::Error> for Replies {
    fn from(error: std::io::Error) -> Replies {
        use std::io::ErrorKind;

        match error.kind() {
            ErrorKind::ConnectionRefused => Replies::ConnectionRefused,
            ErrorKind::ConnectionAborted => Replies::HostUnreachable,
            _ => Replies::NetworkUnreachable,
        }
    }
}

impl From<Replies> for Error {
    fn from(reply: Replies) -> Error {
        let message = match reply {
            Replies::Succeeded => "succeeded",
            Replies::GeneralFailure => "general failure",
            Replies::ConnectionNotAllowed => "connection not allowed",
            Replies::NetworkUnreachable => "network unreachable",
            Replies::HostUnreachable => "host unreachable",
            Replies::ConnectionRefused => "connection refused",
            Replies::TtlExpired => "TTL expired",
            Replies::CommandNotSupported => "command not supported",
            Replies::AddressTypeNotSupported => "address type not supported",
        };
        Error::new(reply, message)
    }
}
