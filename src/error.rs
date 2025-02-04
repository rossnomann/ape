use std::{
    error::Error as StdError, fmt, io::Error as IoError, num::ParseIntError, result::Result as StdResult,
    str::Utf8Error,
};

/// A specialized Result type for metadata operations.
pub type Result<T> = StdResult<T, Error>;

/// Describes all errors that may occur.
#[derive(Debug)]
pub enum Error {
    /// Invalid APE version. It works with APEv2 tags only.
    InvalidApeVersion,
    /// Item keys can have a length of 2 (including) up to 255 (including) characters.
    InvalidItemKeyLen,
    /// Item key contains non-ascii characters.
    InvalidItemKeyValue,
    /// Unexpected item type given while parsing a tag.
    InvalidItemType(u32),
    /// APE header contains invalid tag size.
    InvalidTagSize,
    /// An IO error occured.
    Io(IoError),
    /// Not allowed are the following keys: ID3, TAG, OggS and MP+.
    ItemKeyDenied,
    /// Failed to parse an item key.
    ParseItemKey(Utf8Error),
    /// Can not convert a value of an item with binary type to an UTF-8 string.
    ParseItemBinary,
    /// Failed to parse an item value.
    ParseItemValue(Utf8Error),
    /// Failed to parse Lyrics3V2 size.
    ParseLyrics3V2SizeStr(Utf8Error),
    /// Failed to parse Lyrics3V2 size.
    ParseLyrics3V2SizeInt(ParseIntError),
    /// There is no APE tag in a file.
    TagNotFound,
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(match *self {
            Self::Io(ref err) => err,
            Self::ParseItemKey(ref err) => err,
            Self::ParseItemValue(ref err) => err,
            Self::ParseLyrics3V2SizeStr(ref err) => err,
            Self::ParseLyrics3V2SizeInt(ref err) => err,
            _ => return None,
        })
    }
}

impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::InvalidApeVersion => write!(out, "invalid APE version"),
            Self::InvalidItemKeyLen => write!(out, "item keys can have a length of 2 up to 255 characters"),
            Self::InvalidItemKeyValue => write!(out, "item key contains non-ascii characters"),
            Self::InvalidItemType(value) => write!(out, "invalid item type: {value}"),
            Self::InvalidTagSize => write!(out, "APE header contains invalid tag size"),
            Self::Io(ref err) => write!(out, "{err}"),
            Self::ItemKeyDenied => write!(out, "not allowed are the following keys: ID3, TAG, OggS and MP+"),
            Self::ParseItemKey(ref err) => write!(out, "parse item key: {err}"),
            Self::ParseItemBinary => write!(out, "can not convert a binary value to an UTF-8 string"),
            Self::ParseItemValue(ref err) => write!(out, "parse item value: {err}"),
            Self::ParseLyrics3V2SizeStr(ref err) => write!(out, "parse Lyrics3V2 size: {}", err),
            Self::ParseLyrics3V2SizeInt(ref err) => write!(out, "parse Lyrics3V2 size: {}", err),
            Self::TagNotFound => write!(out, "APE tag does not exist"),
        }
    }
}

impl From<IoError> for Error {
    fn from(error: IoError) -> Self {
        Self::Io(error)
    }
}
