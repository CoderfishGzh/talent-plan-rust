use crate::WalkDir;
use std::io;
use std::io::Error;
use std::net::AddrParseError;
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

    #[fail(display = "{}", _0)]
    File(#[cause] FileError),

    #[fail(display = "Unknown command error")]
    UnKnownCommandError,

    #[fail(display = "invaild addr")]
    AddrError,

    #[fail(display = "must choose the true engine")]
    ErrorEngine,
}

#[derive(Debug, Fail)]
pub enum EntryError {
    #[fail(display = "illegal entry")]
    Illegal,
}

#[derive(Debug, Fail)]
pub enum FileError {
    #[fail(display = "Path Illegal")]
    PathIllegal,

    #[fail(display = "{}", _0)]
    WalkError(#[cause] walkdir::Error),

    #[fail(display = "index file's nums not eq the immutable file's num")]
    OpenFileNumsError,

    #[fail(display = "Illegal offset")]
    IllegalOffset,
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

impl From<walkdir::Error> for KvsError {
    fn from(e: walkdir::Error) -> Self {
        KvsError::File(FileError::WalkError(e))
    }
}

impl From<AddrParseError> for KvsError {
    fn from(_: AddrParseError) -> Self {
        KvsError::AddrError
    }
}
