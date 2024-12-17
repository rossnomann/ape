use crate::error::{Error, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Cursor, Write};

pub const KIND_BINARY: u32 = 1;
pub const KIND_LOCATOR: u32 = 2;
pub const KIND_TEXT: u32 = 0;

const DENIED_KEYS: [&str; 4] = ["ID3", "TAG", "OggS", "MP+"];

/// Represents an [APE Item Value][1]
///
/// [1]: http://wiki.hydrogenaud.io/index.php?title=APE_Item_Value
#[derive(Clone, Debug)]
pub enum ItemValue {
    /// Binary data. Unrecommended to use.
    Binary(Vec<u8>),
    /// Locator is an UTF-8 string contains a link to external stored information.
    Locator(String),
    /// UTF-8 string contains any Text
    Text(String),
}

/// Represents an [APE Tag Item][1].
///
/// [1]: http://wiki.hydrogenaud.io/index.php?title=APE_Tag_Item
#[derive(Clone, Debug)]
pub struct Item {
    /// Item key for accessing special meta-information in an audio file.
    ///
    /// APE tag item keys can have a length of 2 (including) up to 255 (including) characters
    /// in the range from 0x20 (Space) until 0x7E (Tilde).
    ///
    /// Typical keys should have a length of 2 ... 16 characters using the following characters:
    /// Space (0x20), Slash (0x2F), Digits (0x30...0x39), Letters (0x41...0x5A, 0x61...0x7A).
    ///
    /// Not allowed are the following keys: ID3, TAG, OggS and MP+.
    ///
    /// Read the [specification][1] for more information.
    ///
    /// [1]: http://wiki.hydrogenaud.io/index.php?title=APE_key
    pub key: String,
    /// Represents an [APE Item Value][1]
    ///
    /// [1]: http://wiki.hydrogenaud.io/index.php?title=APE_Item_Value
    pub value: ItemValue,
}

impl Item {
    fn new<S: Into<String>>(key: S, value: ItemValue) -> Result<Item> {
        let key = key.into();
        let len = key.len();
        if !(2..=255).contains(&len) {
            return Err(Error::InvalidItemKeyLen);
        }
        if DENIED_KEYS.contains(&key.as_str()) {
            return Err(Error::ItemKeyDenied);
        }
        if !key.is_ascii() {
            return Err(Error::InvalidItemKeyValue);
        }
        Ok(Item { key, value })
    }

    /// Creates an item with Binary value.
    pub fn from_binary<K: Into<String>>(key: K, value: Vec<u8>) -> Result<Item> {
        Self::new(key, ItemValue::Binary(value))
    }

    /// Creates an item with Locator value.
    pub fn from_locator<K: Into<String>, V: Into<String>>(key: K, value: V) -> Result<Item> {
        Self::new(key, ItemValue::Locator(value.into()))
    }

    /// Creates an item with Text value.
    pub fn from_text<K: Into<String>, V: Into<String>>(key: K, value: V) -> Result<Item> {
        Self::new(key, ItemValue::Text(value.into()))
    }

    /// Sets a new Binary value.
    pub fn set_binary(&mut self, value: Vec<u8>) {
        self.value = ItemValue::Binary(value);
    }

    /// Sets a new Locator value.
    pub fn set_locator<S: Into<String>>(&mut self, value: S) {
        self.value = ItemValue::Locator(value.into());
    }

    /// Sets a new Text value.
    pub fn set_text<S: Into<String>>(&mut self, value: S) {
        self.value = ItemValue::Text(value.into());
    }

    /// Creates a representation of the item suitable for writing to a file.
    pub(super) fn to_vec(&self) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let size: u32;
        let flags: u32;
        let value: &[u8];
        match self.value {
            ItemValue::Binary(ref val) => {
                size = val.len() as u32;
                flags = KIND_BINARY << 1;
                value = val;
            }
            ItemValue::Locator(ref val) => {
                size = val.len() as u32;
                flags = KIND_LOCATOR << 1;
                value = val.as_ref();
            }
            ItemValue::Text(ref val) => {
                size = val.len() as u32;
                flags = KIND_TEXT << 1;
                value = val.as_ref();
            }
        };
        cursor.write_u32::<LittleEndian>(size)?;
        cursor.write_u32::<LittleEndian>(flags)?;
        cursor.write_all(self.key.as_ref())?;
        cursor.write_u8(0)?;
        cursor.write_all(value)?;
        Ok(cursor.into_inner())
    }
}

#[cfg(test)]
mod test {
    use super::{Item, ItemValue, DENIED_KEYS, KIND_BINARY, KIND_LOCATOR, KIND_TEXT};
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::{Cursor, Read};

    #[test]
    fn new_failed_with_bad_key_len() {
        let err = Item::from_text("k", "val").unwrap_err().to_string();
        assert_eq!(err, "item keys can have a length of 2 up to 255 characters");
    }

    #[test]
    fn new_failed_with_denied_key() {
        let msg = "not allowed are the following keys: ID3, TAG, OggS and MP+";
        for key in DENIED_KEYS.iter() {
            match Item::from_text((*key).to_string(), "val") {
                Err(err) => {
                    assert_eq!(msg, format!("{err}"));
                }
                Ok(_) => {
                    panic!("Unexpected item");
                }
            };
        }
    }

    #[test]
    fn new_failed_with_bad_key_val() {
        let err = Item::from_text("Недопустимые символы", "val").unwrap_err().to_string();
        assert_eq!(err, "item key contains non-ascii characters");
    }

    #[test]
    fn binary() {
        let vec: Vec<u8> = vec![1];
        let mut item = Item::from_binary("key", vec).unwrap();
        assert_eq!("key", item.key);
        assert_eq!(
            1,
            match item.value {
                ItemValue::Binary(ref val) => val,
                _ => panic!("Invalid value"),
            }[0]
        );
        let vec: Vec<u8> = vec![0];
        item.set_binary(vec);
        assert_eq!(
            0,
            match item.value {
                ItemValue::Binary(ref val) => val,
                _ => panic!("Invalid value"),
            }[0]
        );
    }

    #[test]
    fn locator() {
        let locator = "http://hostname.com";
        let mut item = Item::from_locator("key", locator).unwrap();
        assert_eq!("key", item.key);
        assert_eq!(
            locator,
            match item.value {
                ItemValue::Locator(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );
        let locator = "http://another-hostname.com";
        item.set_locator(locator);
        assert_eq!(
            locator,
            match item.value {
                ItemValue::Locator(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );
    }

    #[test]
    fn text() {
        let text = "text";
        let mut item = Item::from_text("key", text).unwrap();
        assert_eq!("key", item.key);
        assert_eq!(
            text,
            match item.value {
                ItemValue::Text(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );
        let text = "another-text";
        item.set_text(text);
        assert_eq!(
            text,
            match item.value {
                ItemValue::Text(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );
    }

    #[test]
    fn to_vec() {
        let mut data = Cursor::new(Item::from_binary("cover", vec![1, 2, 3]).unwrap().to_vec().unwrap());
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(3, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(KIND_BINARY, (item_flags & 6) >> 1);
        let mut item_key = Vec::<u8>::new();
        let mut k = data.read_u8().unwrap();
        while k != 0 {
            item_key.push(k);
            k = data.read_u8().unwrap();
        }
        assert_eq!("cover", item_key.iter().map(|&c| c as char).collect::<String>());
        let mut item_value = Vec::<u8>::with_capacity(item_size as usize);
        data.take(item_size as u64).read_to_end(&mut item_value).unwrap();
        assert_eq!(vec![1, 2, 3], item_value);

        let mut data = Cursor::new(Item::from_text("artist", "Artist").unwrap().to_vec().unwrap());
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(6, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(KIND_TEXT, (item_flags & 6) >> 1);

        let mut data = Cursor::new(Item::from_locator("url", "http://test.com").unwrap().to_vec().unwrap());
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(15, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(KIND_LOCATOR, (item_flags & 6) >> 1);
    }
}
