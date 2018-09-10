use std::error::Error as StdError;
use std::fmt;
use std::io::Error as IoError;
use std::num::ParseIntError;
use std::result::Result as StdResult;
use std::str::Utf8Error;

/// A specialized Result type for metadata operations.
pub type Result<T> = StdResult<T, Error>;

/// Describes all errors that may occur.
pub enum Error {
    /// An IO error occured. Contains `std::io::Error`.
    Io(IoError),
    /// An error when attempting to interpret a sequence of u8 as a string.
    FromUtf8(Utf8Error),
    /// An error when parsing an integer. Contains `std::num::ParseIntError`.
    ParseInt(ParseIntError),
    /// Unexpected item kind given while parsing a tag.
    BadItemKind,
    /// APE header contains invalid tag size.
    BadTagSize,
    /// Unable to write a tag without items.
    EmptyTag,
    /// Invalid APE version. It works with APEv2 tags only.
    InvalidApeVersion,
    /// Item keys can have a length of 2 (including) up to 255 (including) characters.
    InvalidItemKeyLen,
    /// Item key contains non-ascii characters.
    InvalidItemKeyValue,
    /// Not allowed are the following keys: ID3, TAG, OggS and MP+.
    ItemKeyDenied,
    /// There is no APE tag in a file.
    TagNotFound,
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::ParseInt(ref err) => err.description(),
            Error::FromUtf8(ref err) => err.description(),
            Error::BadItemKind => "Unexpected item kind",
            Error::BadTagSize => "APE header contains invalid tag size",
            Error::EmptyTag => "Unable to perform operations on empty tag",
            Error::InvalidApeVersion => "Invalid APE version",
            Error::InvalidItemKeyLen => "Item keys can have a length of 2 up to 255 characters",
            Error::InvalidItemKeyValue => "Item key contains non-ascii characters",
            Error::ItemKeyDenied => "Not allowed are the following keys: ID3, TAG, OggS and MP+",
            Error::TagNotFound => "APE tag does not exists",
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "{}", self.description())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        write!(out, "{}", self.description())
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Error {
        Error::Io(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Error {
        Error::ParseInt(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Error {
        Error::FromUtf8(error)
    }
}
