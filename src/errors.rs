use std::error::Error as StdError;
use std::fmt;

use http;
use hyper;

pub type Result<T> = ::std::result::Result<T, Error>;

pub enum ErrorKind {
    Hyper(hyper::Error),
    Http(http::Error),
}

pub struct Error {
    kind: ErrorKind,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match &self.kind {
            ErrorKind::Hyper(e) => e.description(),
            ErrorKind::Http(e) => e.description(),
        }
    }
    fn cause(&self) -> Option<&StdError> {
        match &self.kind {
            ErrorKind::Hyper(e) => e.cause(),
            ErrorKind::Http(e) => e.cause(),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Hyper(e) => write!(f, "Hyper Error: {:?}", e),
            ErrorKind::Http(e) => write!(f, "Http Error: {:?}", e),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Hyper(e) => write!(f, "Hyper Error: {}", e),
            ErrorKind::Http(e) => write!(f, "Http Error: {}", e),
        }
    }
}

impl From<hyper::Error> for Error {
    fn from(other: hyper::Error) -> Self {
        Error {
            kind: ErrorKind::Hyper(other),
        }
    }
}

impl From<http::Error> for Error {
    fn from(other: http::Error) -> Self {
        Error {
            kind: ErrorKind::Http(other),
        }
    }
}
