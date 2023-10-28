use std::{
    array::TryFromSliceError,
    fmt::{Debug, Display, Formatter},
    net::AddrParseError,
};

use crate::message::Replies;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Error {
    /// Reply code
    pub reply: Replies,
    /// Error message
    message: String,
}

impl Error {
    pub fn new<S: ToString>(reply: Replies, message: S) -> Error {
        Error {
            reply,
            message: message.to_string(),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(err: TryFromSliceError) -> Error {
        Error::new(Replies::GeneralFailure, err.to_string())
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error {
        Error::new(Replies::GeneralFailure, err.to_string())
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new(Replies::GeneralFailure, err.to_string())
    }
}
