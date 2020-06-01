use std::{
    array::TryFromSliceError,
    fmt::{Debug, Display, Formatter, Result},
    net::AddrParseError,
};

#[derive(Clone)]
pub struct Error {
    /// Reply code
    pub reply: crate::Replies,
    /// Error message
    message: String,
}

impl Error {
    pub fn new<S: ToString>(reply: crate::Replies, message: S) -> Error {
        Error {
            reply,
            message: message.to_string(),
        }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.message)
    }
}

impl From<TryFromSliceError> for Error {
    fn from(err: TryFromSliceError) -> Error {
        Error::new(crate::Replies::GeneralFailure, err.to_string())
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Error {
        Error::new(crate::Replies::GeneralFailure, err.to_string())
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new(crate::Replies::GeneralFailure, err.to_string())
    }
}
