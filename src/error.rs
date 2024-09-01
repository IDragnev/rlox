use std::{
    fmt::Debug,
};
use rlox::scanner::ScanError;

pub enum Error {
    IO(std::io::Error),
    ScanError(ScanError),
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IO(e)
    }
}

impl From<rlox::scanner::ScanError> for Error {
    fn from(e: rlox::scanner::ScanError) -> Self {
        Error::ScanError(e)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(e) => write!(f, "{}", e),
            Self::ScanError(e) => write!(f, "{:?}", e),
        }
    }
}
