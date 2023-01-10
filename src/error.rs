use std::{
    error::Error as StdError, fmt, io::Error as IoError, num::ParseIntError, result::Result as StdResult,
    str::Utf8Error,
};

/// A specialized Result type for metadata operations.
pub type Result<T> = StdResult<T, Error>;

/// Describes all errors that may occur.
#[derive(Debug)]
pub enum Error {
    /// An IO error occured.
    Io(IoError),
    /// An error when attempting to interpret a sequence of u8 as a string.
    FromUtf8(Utf8Error),
    /// An error when parsing an integer.
    ParseInt(ParseIntError),
    /// Unexpected item kind given while parsing a tag.
    BadItemKind,
    /// APE header contains invalid tag size.
    BadTagSize,
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
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::ParseInt(ref err) => Some(err),
            _ => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(out, "{err}"),
            Error::ParseInt(ref err) => write!(out, "{err}"),
            Error::FromUtf8(ref err) => write!(out, "{err}"),
            Error::BadItemKind => write!(out, "unexpected item kind"),
            Error::BadTagSize => write!(out, "APE header contains invalid tag size"),
            Error::InvalidApeVersion => write!(out, "invalid APE version"),
            Error::InvalidItemKeyLen => write!(out, "item keys can have a length of 2 up to 255 characters"),
            Error::InvalidItemKeyValue => write!(out, "item key contains non-ascii characters"),
            Error::ItemKeyDenied => write!(out, "not allowed are the following keys: ID3, TAG, OggS and MP+"),
            Error::TagNotFound => write!(out, "APE tag does not exists"),
        }
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
