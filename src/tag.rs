extern crate byteorder;

use std::io::{Read, Seek, SeekFrom, Write};
use std::fs::{File, OpenOptions};
use std::path::Path;

use self::byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use error::{Error, Result};
use item::{Item, KIND_BINARY, KIND_LOCATOR, KIND_TEXT};
use meta::{Meta, APE_VERSION};
use util::{APE_PREAMBLE, probe_id3v1, probe_lyrics3v2, vec_to_string};

const BUFFER_SIZE: u64 = 65536;

/// An APE Tag containing APE Tag Items.
/// # Examples
///
/// ## Creating a tag
///
/// ```no_run
/// use ape::{Tag, Item};
///
/// let mut tag = Tag::new();
/// let item = Item::from_text("artist", "Artist Name").unwrap();
/// tag.set_item(item);
/// tag.write("path/to/file").unwrap();
/// ```
/// # Updating a tag
///
/// ```no_run
/// use ape::{read, Item};
///
/// let path = "path/to/file";
/// let mut tag = read(path).unwrap();
/// let item = Item::from_text("album", "Album Name").unwrap();
/// tag.set_item(item);
/// tag.remove_item("cover");
/// tag.write(path).unwrap();
/// ```
#[derive(Debug)]
pub struct Tag {
    /// A vector of items included in the tag.
    pub items: Vec<Item>,
}

impl Tag {
    /// Creates a new empty tag.
    pub fn new() -> Tag {
        Tag { items: Vec::new() }
    }

    /// Returns an item by key.
    pub fn item(&self, key: &str) -> Option<&Item> {
        let key = key.to_string();
        self.items.iter()
                  .position(|item| item.key == key)
                  .map(|idx| self.items.get(idx).unwrap())
    }

    /// Sets a new item.
    ///
    /// If there is an item with the same key, it will be removed.
    pub fn set_item(&mut self, item: Item) {
        self.remove_item(item.key.as_ref());
        self.items.push(item);
    }

    /// Removes an item by key.
    ///
    /// Returns true, if item was removed, and false otherwise.
    pub fn remove_item(&mut self, key: &str) -> bool {
        let key = key.to_string();
        self.items.iter()
                  .position(|item| item.key == key)
                  .map(|idx| self.items.remove(idx))
                  .is_some()
    }

    /// Attempts to write the APE Tag to the file at the specified path.
    ///
    /// # Errors
    ///
    /// It is considered an error if there are no items in the tag.
    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if self.items.len() == 0 {
            return Err(Error::EmptyTag);
        }

        try!(remove(&path));

        let mut file = &try!(OpenOptions::new().read(true).write(true).open(path));

        // Keep ID3v1 and LYRICS3v2 (if any)
        let mut id3 = Vec::<u8>::new();
        let filesize = try!(file.seek(SeekFrom::End(0)));
        if try!(probe_id3v1(&mut file)) {
            let mut end_size: i64 = 128;
            let lyrcis3v2_size = try!(probe_lyrics3v2(&mut file));
            if lyrcis3v2_size != -1 {
                end_size += lyrcis3v2_size;
            }
            try!(file.seek(SeekFrom::End(-end_size)));
            try!(file.take(end_size as u64).read_to_end(&mut id3));
            try!(file.seek(SeekFrom::End(-end_size)));
            try!(file.set_len(filesize - end_size as u64));
        }
        try!(file.seek(SeekFrom::End(0)));

        // Convert items to bytes
        let mut items = Vec::<Vec<u8>>::new();
        for item in &self.items {
            items.push(try!(item.to_vec()));
        }
        // APE tag items should be sorted ascending by size
        items.sort_by(|a, b| a.len().cmp(&b.len()));
        let mut size = 32; // Tag size including footer

        // Write items
        for item in items {
            size += item.len();
            try!(file.write_all(&item));
        }

        // Write footer
        try!(file.write_all(APE_PREAMBLE));
        try!(file.write_u32::<LittleEndian>(APE_VERSION));
        // Tag size including footer
        try!(file.write_u32::<LittleEndian>(size as u32));
        // Item count
        try!(file.write_u32::<LittleEndian>(self.items.len() as u32));
        // Tag flags
        try!(file.write_u32::<LittleEndian>(0));
        // Reserved
        for _ in 0..8 {
            try!(file.write_u8(0));
        }

        // Write ID3v1 and LYRICS3v2 (if any)
        try!(file.write_all(&id3));
        Ok(())
    }
}


/// Attempts to read APE tag from the file at the specified path.
///
/// # Errors
///
/// It is considered a error when:
///
/// - APE tag does not exists.
/// - Tag version is not 2.000.
/// - Item key is not valid.
/// - Kind of an item is unknown.
/// - Tag size declared in the APE header does not match with actual size.
///
/// # Examples
///
/// ```no_run
/// use ape::read;
///
/// let tag = read("path/to/file").unwrap();
/// let item = tag.item("artist").unwrap();
/// println!("{:?}", item.value);
/// ```
pub fn read<P: AsRef<Path>>(path: P) -> Result<Tag> {
    let mut file = &try!(File::open(path));
    let meta = try!(Meta::read(&mut file));
    let mut items = Vec::<Item>::new();
    try!(file.seek(SeekFrom::Start(meta.start_pos)));
    for _ in 0..meta.item_count {
        let item_size = try!(file.read_u32::<LittleEndian>());
        let item_flags = try!(file.read_u32::<LittleEndian>());
        let mut item_key = Vec::<u8>::new();
        let mut k = try!(file.read_u8());
        while k != 0 {
            item_key.push(k);
            k = try!(file.read_u8());
        }
        let mut item_value = Vec::<u8>::with_capacity(item_size as usize);
        try!(file.take(item_size as u64).read_to_end(&mut item_value));
        let item_key = vec_to_string(&item_key);
        items.push(
            match (item_flags & 6) >> 1 {
                KIND_BINARY => try!(Item::from_binary(item_key, item_value)),
                KIND_LOCATOR => try!(Item::from_locator(item_key, vec_to_string(&item_value))),
                KIND_TEXT => try!(Item::from_text(item_key, vec_to_string(&item_value))),
                _ => {
                    return Err(Error::BadItemKind);
                }
            }
        );
    }
    if try!(file.seek(SeekFrom::Current(0))) != meta.end_pos {
        Err(Error::BadTagSize)
    } else {
        Ok(Tag{items: items})
    }
}

/// Attempts to remove APE tag from the file at the specified path.
///
/// # Errors
///
/// - It is considered a error when tag version is not 2.000.
/// - It is **not** considered a error when tag does not exists.
///
/// # Examples
///
/// ```no_run
/// use ape::remove;
///
/// remove("path/to/file").unwrap();
/// ```
pub fn remove<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file = &try!(OpenOptions::new().read(true).write(true).open(path));
    let meta = match Meta::read(&mut file) {
        Ok(meta) => meta,
        Err(error) => match error {
            Error::TagNotFound => {
                // It's ok, nothing to remove.
                return Ok(());
            },
            _ => {
                return Err(error);
            }
        }
    };
    let mut size = meta.size as u64;
    let mut offset;
    if meta.is_header {
        offset = 0;
        size += 32;
    } else {
        offset = meta.start_pos;
        if meta.has_header {
            offset -= 32;
            size += 32;
        }
    }
    let filesize = try!(file.seek(SeekFrom::End(0)));
    let movesize = filesize - offset - size;
    if movesize > 0 {
        try!(file.flush());
        try!(file.seek(SeekFrom::Start(offset + size)));
        let mut buff = Vec::<u8>::with_capacity(BUFFER_SIZE as usize);
        try!(file.take(BUFFER_SIZE).read_to_end(&mut buff));
        while buff.len() > 0 {
            try!(file.seek(SeekFrom::Start(offset)));
            try!(file.write(&buff));
            offset += buff.len() as u64;
            try!(file.seek(SeekFrom::Start(offset + size)));
            buff.clear();
            try!(file.take(BUFFER_SIZE).read_to_end(&mut buff));
        }
    }
    try!(file.set_len(filesize - size));
    try!(file.flush());
    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs::{File, remove_file};
    use std::io::Write;
    use item::{Item, ItemValue};
    use super::{Tag, read, remove};

    #[test]
    fn items() {
        let mut tag = Tag::new();
        let item = Item::from_text("key", "value").unwrap();
        assert_eq!(0, tag.items.len());
        tag.set_item(item);
        assert_eq!(1, tag.items.len());
        assert_eq!("value", match tag.item("key").unwrap().value {
            ItemValue::Text(ref val) => val,
            _ => panic!("Invalid value")
        });
        assert!(tag.remove_item("key"));
        assert_eq!(0, tag.items.len());
        assert!(!tag.remove_item("key"));
    }

    #[test]
    fn read_write_remove() {
        let path = "data/read-write-remove.apev2";

        let mut data = File::create(path).unwrap();
        data.write_all(&[0; 200]).unwrap();

        let mut tag = Tag::new();
        tag.set_item(Item::from_text("key", "value").unwrap());
        tag.write(path).unwrap();

        let tag = read(path).unwrap();
        assert_eq!(1, tag.items.len());
        assert_eq!("value", match tag.item("key").unwrap().value {
            ItemValue::Text(ref val) => val,
            _ => panic!("Invalid value")
        });

        remove(path).unwrap();
        match read(path) {
            Err(_) => {},
            Ok(_) => panic!("The tag wasn't removed!")
        };

        remove_file(path).unwrap();
    }

    #[test]
    #[should_panic(expected = "Unable to perform operations on empty tag")]
    fn write_failed_with_empty_tag() {
        Tag::new().write("data/empty").unwrap();
    }

    #[test]
    #[should_panic(expected = "Unexpected item kind")]
    fn read_failed_with_bad_item_kind() {
        read("data/bad-item-kind.apev2").unwrap();
    }

    #[test]
    #[should_panic(expected = "APE header contains invalid tag size")]
    fn read_failed_with_bad_tag_size() {
        read("data/bad-tag-size.apev2").unwrap();
    }

    #[test]
    fn remove_for_no_tag_is_ok() {
        remove("data/no-tag.apev2").unwrap();
    }
}
