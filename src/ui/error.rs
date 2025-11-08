use std::{io, sync::mpsc};

#[derive(Debug)]
pub enum Error {
    IoError(io::Error),
    RecvError(mpsc::RecvError),
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<mpsc::RecvError> for Error {
    fn from(value: mpsc::RecvError) -> Self {
        Error::RecvError(value)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
