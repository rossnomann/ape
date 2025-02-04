use crate::error::{Error, Result};
use byteorder::{LittleEndian, WriteBytesExt};
use std::io::{Cursor, Write};

/// Represents an [APE Tag Item Type][1] (bit 2..1)
///
/// [1]: https://wiki.hydrogenaud.io/index.php?title=Ape_Tags_Flags
#[derive(Clone, Copy, Debug, PartialOrd, PartialEq)]
pub enum ItemType {
    /// Item contains binary information
    Binary,
    /// Item is a locator of external stored information
    Locator,
    /// Item contains text information coded in UTF-8
    Text,
}

impl ItemType {
    pub(super) fn from_flags(item_flags: u32) -> Result<Self> {
        Ok(match (item_flags & 6) >> 1 {
            1 => Self::Binary,
            2 => Self::Locator,
            0 => Self::Text,
            _ => return Err(Error::BadItemType),
        })
    }

    fn as_u32(&self) -> u32 {
        match self {
            Self::Binary => 1u32,
            Self::Locator => 2u32,
            Self::Text => 0u32,
        }
    }
}

const DENIED_KEYS: [&str; 4] = ["ID3", "TAG", "OggS", "MP+"];

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
    item_type: ItemType,
    /// Represents an [APE Item Value][1]
    ///
    /// [1]: http://wiki.hydrogenaud.io/index.php?title=APE_Item_Value
    item_value: Vec<u8>,
}

impl Item {
    /// Creates a new `Item`.
    pub fn new<K: Into<String>, V: Into<Vec<u8>>>(key: K, item_type: ItemType, item_value: V) -> Result<Self> {
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
        Ok(Self {
            key,
            item_type,
            item_value: item_value.into(),
        })
    }

    /// Returns a type of the item.
    pub fn get_type(&self) -> ItemType {
        self.item_type
    }

    /// Adds a new value separated by `\0`.
    pub fn add_value(&mut self, value: &[u8]) {
        self.item_value.push(0);
        self.item_value.extend(value);
    }

    /// Replaces a type.
    pub fn with_type(mut self, item_type: ItemType) -> Self {
        self.item_type = item_type;
        self
    }

    /// Replaces a value.
    pub fn with_value<V: Into<Vec<u8>>>(mut self, value: V) -> Self {
        self.item_value = value.into();
        self
    }

    /// Creates a representation of the item suitable for writing to a file.
    pub(super) fn to_vec(&self) -> Result<Vec<u8>> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        let flags: u32 = self.item_type.as_u32() << 1;
        let size: u32 = self.item_value.len() as u32;
        cursor.write_u32::<LittleEndian>(size)?;
        cursor.write_u32::<LittleEndian>(flags)?;
        cursor.write_all(self.key.as_ref())?;
        cursor.write_u8(0)?;
        cursor.write_all(&self.item_value)?;
        Ok(cursor.into_inner())
    }
}

impl<'a> TryFrom<&'a Item> for &'a str {
    type Error = Error;

    fn try_from(value: &'a Item) -> Result<Self> {
        std::str::from_utf8(&value.item_value).map_err(Error::ParseItemValue)
    }
}

impl TryFrom<Item> for String {
    type Error = Error;

    fn try_from(value: Item) -> Result<Self> {
        let result = std::str::from_utf8(&value.item_value).map_err(Error::ParseItemValue)?;
        Ok(String::from(result))
    }
}

impl<'a> TryFrom<&'a Item> for Vec<&'a str> {
    type Error = Error;

    fn try_from(value: &'a Item) -> Result<Self> {
        let mut result = Vec::with_capacity(value.item_value.len());
        for x in value.item_value.split(|&c| c == 0) {
            let x = std::str::from_utf8(x).map_err(Error::ParseItemValue)?;
            result.push(x);
        }
        Ok(result)
    }
}

impl TryFrom<Item> for Vec<String> {
    type Error = Error;

    fn try_from(value: Item) -> Result<Self> {
        let mut result = Vec::with_capacity(value.item_value.len());
        for x in value.item_value.split(|&c| c == 0) {
            let x = std::str::from_utf8(x).map_err(Error::ParseItemValue)?;
            result.push(String::from(x));
        }
        Ok(result)
    }
}

impl From<&Item> for Vec<u8> {
    fn from(value: &Item) -> Self {
        value.item_value.clone()
    }
}

impl From<Item> for Vec<u8> {
    fn from(value: Item) -> Self {
        value.item_value
    }
}

#[cfg(test)]
mod test {
    use super::{Item, ItemType, DENIED_KEYS};
    use byteorder::{LittleEndian, ReadBytesExt};
    use std::io::{Cursor, Read};

    #[test]
    fn new_failed_with_bad_key_len() {
        let err = Item::new("k", ItemType::Text, "val").unwrap_err().to_string();
        assert_eq!(err, "item keys can have a length of 2 up to 255 characters");
    }

    #[test]
    fn new_failed_with_denied_key() {
        let msg = "not allowed are the following keys: ID3, TAG, OggS and MP+";
        for key in DENIED_KEYS.iter() {
            match Item::new((*key).to_string(), ItemType::Text, "val") {
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
        let err = Item::new("Недопустимые символы", ItemType::Text, "val")
            .unwrap_err()
            .to_string();
        assert_eq!(err, "item key contains non-ascii characters");
    }

    #[test]
    fn construct() {
        let initial_value: Vec<u8> = vec![1];
        let mut item = Item::new("key", ItemType::Binary, initial_value.clone()).unwrap();
        assert_eq!("key", item.key);
        assert_eq!(ItemType::Binary, item.item_type);
        assert_eq!(item.get_type(), item.item_type);
        assert_eq!(initial_value, item.item_value);
        item.add_value(String::from("x").as_ref());
        assert_eq!(vec![1, 0, 120], item.item_value);

        let new_value = String::from("test");
        let item = item.with_type(ItemType::Text).with_value(new_value.clone());
        assert_eq!(ItemType::Text, item.item_type);
        assert_eq!(Vec::from(new_value), item.item_value);
    }

    #[test]
    fn to_vec() {
        let mut data = Cursor::new(
            Item::new("cover", ItemType::Binary, vec![1, 2, 3])
                .unwrap()
                .to_vec()
                .unwrap(),
        );
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(3, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(ItemType::Binary.as_u32(), (item_flags & 6) >> 1);
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

        let mut data = Cursor::new(Item::new("artist", ItemType::Text, "Artist").unwrap().to_vec().unwrap());
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(6, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(ItemType::Text.as_u32(), (item_flags & 6) >> 1);

        let mut data = Cursor::new(
            Item::new("url", ItemType::Locator, "http://test.com")
                .unwrap()
                .to_vec()
                .unwrap(),
        );
        let item_size = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(15, item_size);
        let item_flags = data.read_u32::<LittleEndian>().unwrap();
        assert_eq!(ItemType::Locator.as_u32(), (item_flags & 6) >> 1);
    }
}
