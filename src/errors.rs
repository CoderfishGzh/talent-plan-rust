use std::io;
use std::io::Error;
use std::string::FromUtf8Error;

pub type Result<T> = std::result::Result<T, KvsError>;

#[derive(Debug, Fail)]
pub enum KvsError {
    #[fail(display = "{}", _0)]
    Io(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    SerdeJ(#[cause] serde_json::Error),

    #[fail(display = "{}", _0)]
    EntryError(#[cause] EntryError),

    #[fail(display = "Key not found")]
    KeyNotExit,

    #[fail(display = "File end")]
    FileEnd,

    #[fail(display = "{}", _0)]
    Utf8Error(#[cause] FromUtf8Error),
}

#[derive(Debug, Fail)]
pub enum EntryError {
    #[fail(display = "illegal entry")]
    Illegal,
}

impl From<io::Error> for KvsError {
    fn from(e: Error) -> Self {
        KvsError::Io(e)
    }
}

impl From<FromUtf8Error> for KvsError {
    fn from(e: FromUtf8Error) -> Self {
        KvsError::Utf8Error(e)
    }
}

impl From<serde_json::Error> for KvsError {
    fn from(e: serde_json::Error) -> Self {
        KvsError::SerdeJ(e)
    }
}
