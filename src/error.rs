use std::fmt::{Debug, Display, Formatter, Result};

#[derive(Clone)]
pub struct Error {
    /// Reply code
    pub reply: crate::Replies,
    /// Error message
    message: String,
}

impl Error {
    pub fn new<S: Into<String>>(reply: crate::Replies, message: S) -> Error {
        Error {
            reply,
            message: message.into(),
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

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::new(crate::Replies::GeneralFailure, err.to_string())
    }
}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, err.message)
    }
}
