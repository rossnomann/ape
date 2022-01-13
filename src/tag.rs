use crate::{
    error::{Error, Result},
    item::{Item, KIND_BINARY, KIND_LOCATOR, KIND_TEXT},
    meta::{Meta, MetaPosition, APE_VERSION},
    util::{probe_id3v1, probe_lyrics3v2, APE_PREAMBLE},
};
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    slice::Iter as SliceIter,
    str,
    vec::IntoIter as VecIntoIter,
};

/// An APE Tag containing APE Tag Items.
///
/// # Examples
///
/// ## Creating a tag
///
/// ```no_run
/// use ape::{Item, Tag, write_to_path};
///
/// let mut tag = Tag::new();
/// let item = Item::from_text("artist", "Artist Name").unwrap();
/// tag.set_item(item);
/// write_to_path(&tag, "path/to/file").unwrap();
/// ```
///
/// ## Updating a tag
///
/// ```no_run
/// use ape::{Item, read_from_path, write_to_path};
///
/// let path = "path/to/file";
/// let mut tag = read_from_path(path).unwrap();
/// let item = Item::from_text("album", "Album Name").unwrap();
/// tag.set_item(item);
/// tag.remove_item("cover");
/// write_to_path(&tag, path).unwrap();
/// ```
#[derive(Debug, Default)]
pub struct Tag(Vec<Item>);

impl Tag {
    /// Creates a new empty tag.
    pub fn new() -> Tag {
        Self::default()
    }

    /// Returns an item by key.
    pub fn item(&self, key: &str) -> Option<&Item> {
        self.0
            .iter()
            .find(|item| item.key.eq_ignore_ascii_case(key))
    }

    /// Sets a new item.
    ///
    /// If there is an item with the same key, it will be removed.
    pub fn set_item(&mut self, item: Item) {
        self.remove_item(item.key.as_ref());
        self.0.push(item);
    }

    /// Removes an item by key.
    ///
    /// Returns true, if item was removed, and false otherwise.
    pub fn remove_item(&mut self, key: &str) -> bool {
        self.0
            .iter()
            .position(|item| item.key.eq_ignore_ascii_case(key))
            .map(|idx| self.0.remove(idx))
            .is_some()
    }

    /// Returns an iterator over the tag
    pub fn iter(&self) -> SliceIter<Item> {
        self.0.iter()
    }
}

impl IntoIterator for Tag {
    type Item = Item;
    type IntoIter = VecIntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Attempts to write the APE tag to the file at the specified path.
pub fn write_to_path<P: AsRef<Path>>(tag: &Tag, path: P) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    write_to(tag, &mut file)?;

    Ok(())
}

/// Attempts to write the APE tag to a File.
pub fn write_to(tag: &Tag, file: &mut File) -> Result<()> {
    // Convert items to bytes
    // Do it as early as possible because if there is any error,
    // we return it without modifying the file
    let mut items = tag.iter().map(|item| item.to_vec()).collect::<Result<Vec<_>>>()?;

    // APE tag items should be sorted ascending by size
    items.sort_by_key(|a| a.len());

    remove_from(file)?;

    // Keep ID3v1 and LYRICS3v2 (if any)
    let mut id3 = Vec::<u8>::new();
    let filesize = file.seek(SeekFrom::End(0))?;

    if probe_id3v1(file)? {
        let mut end_size: i64 = 128;
        let lyrcis3v2_size = probe_lyrics3v2(file)?;

        if lyrcis3v2_size != -1 {
            end_size += lyrcis3v2_size;
        }

        file.seek(SeekFrom::End(-end_size))?;
        file.take(end_size as u64).read_to_end(&mut id3)?;
        file.seek(SeekFrom::End(-end_size))?;
        file.set_len(filesize - end_size as u64)?;
    }

    file.seek(SeekFrom::End(0))?;

    let mut size = 32; // Tag size including footer

    // Write items
    for item in items {
        size += item.len();
        file.write_all(&item)?;
    }

    // Write footer
    file.write_all(APE_PREAMBLE)?;
    file.write_u32::<LittleEndian>(APE_VERSION)?;
    // Tag size including footer
    file.write_u32::<LittleEndian>(size as u32)?;
    // Item count
    file.write_u32::<LittleEndian>(tag.0.len() as u32)?;
    // Tag flags
    file.write_u32::<LittleEndian>(0)?;

    // Reserved
    for _ in 0..8 {
        file.write_u8(0)?;
    }

    // Write ID3v1 and LYRICS3v2 (if any)
    file.write_all(&id3)?;

    Ok(())
}

/// Attempts to read an APE tag from the file at the specified path.
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
/// use ape::read_from_path;
///
/// let tag = read_from_path("path/to/file").unwrap();
/// let item = tag.item("artist").unwrap();
/// println!("{:?}", item.value);
/// ```
pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Tag> {
    let mut file = OpenOptions::new().read(true).open(path)?;
    read_from(&mut file)
}

/// Attempts to read an APE tag from a reader
///
/// # Errors
///
/// See [`read_from_path`](fn.read_from_path.html)
pub fn read_from<R: Read + Seek>(reader: &mut R) -> Result<Tag> {
    let meta = Meta::read(reader)?;
    let mut items = Vec::<Item>::new();

    reader.seek(SeekFrom::Start(meta.start_pos))?;

    for _ in 0..meta.item_count {
        let item_size = reader.read_u32::<LittleEndian>()?;
        let item_flags = reader.read_u32::<LittleEndian>()?;
        let mut item_key = Vec::<u8>::new();
        let mut k = reader.read_u8()?;

        while k != 0 {
            item_key.push(k);
            k = reader.read_u8()?;
        }

        let mut item_value = Vec::<u8>::with_capacity(item_size as usize);
        reader.take(item_size as u64).read_to_end(&mut item_value)?;

        let item_key = str::from_utf8(&item_key)?;
        items.push(match (item_flags & 6) >> 1 {
            KIND_BINARY => Item::from_binary(item_key, item_value)?,
            KIND_LOCATOR => Item::from_locator(item_key, str::from_utf8(&item_value)?)?,
            KIND_TEXT => Item::from_text(item_key, str::from_utf8(&item_value)?)?,
            _ => {
                return Err(Error::BadItemKind);
            }
        });
    }

    if reader.seek(SeekFrom::Current(0))? != meta.end_pos {
        Err(Error::BadTagSize)
    } else {
        Ok(Tag(items))
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
/// use ape::remove_from_path;
///
/// remove_from_path("path/to/file").unwrap();
/// ```
pub fn remove_from_path<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut file = OpenOptions::new().read(true).write(true).open(path)?;
    remove_from(&mut file)?;

    Ok(())
}

/// Attempts to remove an APE tag from a File
///
/// # Errors
///
/// See [`remove_from_path`](fn.remove_from_path.html)
pub fn remove_from(file: &mut File) -> Result<()> {
    let meta = match Meta::read(file) {
        Ok(meta) => meta,
        Err(error) => {
            return match error {
                Error::TagNotFound => {
                    // It's ok, nothing to remove.
                    Ok(())
                }
                _ => Err(error),
            };
        }
    };

    let mut size = meta.size as u64;
    let mut offset;

    match meta.position {
        MetaPosition::Header => {
            offset = 0;
            size += 32;
        }
        MetaPosition::Footer => {
            offset = meta.start_pos;
            if meta.has_header {
                offset -= 32;
                size += 32;
            }
        }
    }

    let filesize = file.seek(SeekFrom::End(0))?;
    let movesize = filesize - offset - size;

    const BUFFER_SIZE: u64 = 65536;

    if movesize > 0 {
        file.flush()?;
        file.seek(SeekFrom::Start(offset + size))?;

        let mut buff = Vec::<u8>::with_capacity(BUFFER_SIZE as usize);
        file.take(BUFFER_SIZE as u64).read_to_end(&mut buff)?;

        while !buff.is_empty() {
            file.seek(SeekFrom::Start(offset))?;
            file.write_all(&buff)?;
            offset += buff.len() as u64;
            file.seek(SeekFrom::Start(offset + size))?;
            buff.clear();
            file.take(BUFFER_SIZE as u64).read_to_end(&mut buff)?;
        }
    }

    file.set_len(filesize - size)?;
    file.flush()?;

    Ok(())
}

#[cfg(test)]
mod test {
    use super::{read_from_path, remove_from_path, write_to_path, Tag};
    use crate::item::{Item, ItemValue};
    use std::{
        fs::{remove_file, File},
        io::Write,
    };

    #[test]
    fn items() {
        let mut tag = Tag::new();
        assert_eq!(0, tag.0.len());

        let item = Item::from_text("key", "value").unwrap();

        tag.set_item(item);
        assert_eq!(1, tag.0.len());

        assert_eq!(
            "value",
            match tag.item("key").unwrap().value {
                ItemValue::Text(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );
        assert!(tag.remove_item("key"));
        assert_eq!(0, tag.0.len());
        assert!(!tag.remove_item("key"));
    }

    #[test]
    fn read_write_remove() {
        let path = "data/read-write-remove.apev2";

        let mut data = File::create(path).unwrap();
        data.write_all(&[0; 200]).unwrap();

        let mut tag = Tag::new();
        tag.set_item(Item::from_text("key", "value").unwrap());
        write_to_path(&tag, path).unwrap();

        let tag = read_from_path(path).unwrap();
        assert_eq!(1, tag.0.len());
        assert_eq!(
            "value",
            match tag.item("key").unwrap().value {
                ItemValue::Text(ref val) => val,
                _ => panic!("Invalid value"),
            }
        );

        remove_from_path(path).unwrap();
        match read_from_path(path) {
            Err(_) => {}
            Ok(_) => panic!("The tag wasn't removed!"),
        };

        remove_file(path).unwrap();
    }

    #[test]
    fn read_with_empty_tag() {
        assert!(read_from_path("data/empty-tag.apev2").is_ok());
    }

    #[test]
    fn write_with_empty_tag() {
        assert!(write_to_path(&Tag::new(), "data/empty-tag.apev2").is_ok());
    }

    #[test]
    fn read_failed_with_bad_item_kind() {
        let err = read_from_path("data/bad-item-kind.apev2").unwrap_err().to_string();
        assert_eq!(err, "unexpected item kind");
    }

    #[test]
    fn read_failed_with_bad_tag_size() {
        let err = read_from_path("data/bad-tag-size.apev2").unwrap_err().to_string();
        assert_eq!(err, "APE header contains invalid tag size");
    }

    #[test]
    fn remove_for_no_tag_is_ok() {
        remove_from_path("data/no-tag.apev2").unwrap();
    }
}
