extern crate byteorder;

use std::io::{Read, Seek, SeekFrom};

use self::byteorder::{LittleEndian, ReadBytesExt};

use error::{Error, Result};
use util::{probe_ape, probe_id3v1, probe_lyrics3v2, ID3V1_OFFSET};

pub const APE_VERSION: u32 = 2000;
const APE_HEADER_SIZE: i64 = 32;

const HAS_HEADER: u32 = 1 << 31;
const HAS_NO_FOOTER: u32 = 1 << 30;
const IS_HEADER: u32 = 1 << 29;

#[derive(Debug)]
pub struct Meta {
    // Tag size in bytes including footer and all tag items excluding the header.
    pub size: u32,
    // Number of items in the Tag.
    pub item_count: u32,
    // This is the header, not the footer.
    pub is_header: bool,
    // Tag contains a header.
    pub has_header: bool,
    // Initial position of the Tag items.
    pub start_pos: u64,
    // End position of the Tag items.
    pub end_pos: u64,
}


impl Meta {
    pub fn read<R: Read + Seek>(reader: &mut R) -> Result<Meta> {
        let mut found = try!(probe_ape(reader, SeekFrom::End(-APE_HEADER_SIZE)));
        if !found  {
            found = try!(probe_ape(reader, SeekFrom::Start(0)));
            if !found {
                // When located at the end of an MP3 file, an APE tag should be placed after
                // the the last frame, just before the ID3v1 tag (if any).
                if try!(probe_id3v1(reader)) {
                    found = try!(probe_ape(reader, SeekFrom::End(ID3V1_OFFSET - APE_HEADER_SIZE)));
                    if !found {
                        // ID3v1 tag maybe preceded by Lyrics3v2: http://id3.org/Lyrics3v2
                        let size = try!(probe_lyrics3v2(reader));
                        if size != -1 {
                            found = try!(probe_ape(reader, SeekFrom::End(ID3V1_OFFSET - size - APE_HEADER_SIZE)));
                        }
                    }
                }
            }
        }
        if !found {
            return Err(Error::TagNotFound);
        }
        if try!(reader.read_u32::<LittleEndian>()) != APE_VERSION {
            return Err(Error::InvalidApeVersion);
        }
        let size = try!(reader.read_u32::<LittleEndian>());
        let item_count = try!(reader.read_u32::<LittleEndian>());
        let flags = try!(reader.read_u32::<LittleEndian>());
        // The following 8 bytes are reserved
        let end_pos = try!(reader.seek(SeekFrom::Current(8)));
        let is_header = flags & IS_HEADER != 0;
        Ok(Meta{
            size: size,
            item_count: item_count,
            is_header: is_header,
            has_header: flags & HAS_HEADER != 0,
            start_pos: if is_header {
                end_pos
            } else {
                end_pos - size as u64
            },
            end_pos: if is_header {
                let mut pos = end_pos + size as u64;
                if flags & HAS_NO_FOOTER == 0 {
                    pos -= 32;
                }
                pos
            } else {
                end_pos - 32
            },
        })
    }
}

#[cfg(test)]
mod test {
    extern crate byteorder;
    use std::io::{Cursor, Write};
    use self::byteorder::{LittleEndian, WriteBytesExt};
    use super::{Meta, HAS_HEADER, IS_HEADER, HAS_NO_FOOTER};

    #[test]
    fn found_at_end() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let size = 40;
        let item_count = 4;
        let flags = 0;
        data.write_all(&[0; 100]).unwrap();
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(2000).unwrap();
        data.write_u32::<LittleEndian>(size).unwrap();
        data.write_u32::<LittleEndian>(item_count).unwrap();
        data.write_u32::<LittleEndian>(flags).unwrap();
        data.write_all(&[0; 8]).unwrap();
        let meta = Meta::read(&mut data).unwrap();
        assert_eq!(size, meta.size);
        assert_eq!(item_count, meta.item_count);
        assert!(!meta.is_header);
        assert!(!meta.has_header);
        assert_eq!(92, meta.start_pos);
        assert_eq!(100, meta.end_pos);
    }

    #[test]
    fn found_at_start() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let size = 50;
        let item_count = 5;
        let flags = HAS_HEADER | IS_HEADER | HAS_NO_FOOTER;
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(2000).unwrap();
        data.write_u32::<LittleEndian>(size).unwrap();
        data.write_u32::<LittleEndian>(item_count).unwrap();
        data.write_u32::<LittleEndian>(flags).unwrap();
        data.write_all(&[0; 8]).unwrap();
        data.write_all(&[0; 200]).unwrap();
        let meta = Meta::read(&mut data).unwrap();
        assert_eq!(size, meta.size);
        assert_eq!(item_count, meta.item_count);
        assert!(meta.is_header);
        assert!(meta.has_header);
        assert_eq!(32, meta.start_pos);
        assert_eq!(82, meta.end_pos);
    }

    #[test]
    fn found_before_id3v1() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let size = 62;
        let item_count = 3;
        let flags = 0;
        data.write_all(&[0; 300]).unwrap();
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(2000).unwrap();
        data.write_u32::<LittleEndian>(size).unwrap();
        data.write_u32::<LittleEndian>(item_count).unwrap();
        data.write_u32::<LittleEndian>(flags).unwrap();
        data.write_all(&[0; 8]).unwrap();
        data.write_all(b"TAG").unwrap();
        data.write_all(&[0; 125]).unwrap();
        let meta = Meta::read(&mut data).unwrap();
        assert_eq!(size, meta.size);
        assert_eq!(item_count, meta.item_count);
        assert!(!meta.is_header);
        assert!(!meta.has_header);
        assert_eq!(270, meta.start_pos);
        assert_eq!(300, meta.end_pos);
    }

    #[test]
    fn found_before_lyrics3v2() {
        let mut data = Cursor::new(Vec::<u8>::new());
        let size = 70;
        let item_count = 2;
        let flags = 0;
        data.write_all(&[0; 600]).unwrap();
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(2000).unwrap();
        data.write_u32::<LittleEndian>(size).unwrap();
        data.write_u32::<LittleEndian>(item_count).unwrap();
        data.write_u32::<LittleEndian>(flags).unwrap();
        data.write_all(&[0; 8]).unwrap();
        data.write_all(&[0; 120]).unwrap();
        data.write_all(b"000120LYRICS200").unwrap();
        data.write_all(b"TAG").unwrap();
        data.write_all(&[0; 125]).unwrap();
        let meta = Meta::read(&mut data).unwrap();
        assert_eq!(size, meta.size);
        assert_eq!(item_count, meta.item_count);
        assert!(!meta.is_header);
        assert!(!meta.has_header);
        assert_eq!(562, meta.start_pos);
        assert_eq!(600, meta.end_pos);
    }

    #[test]
    #[should_panic(expected = "APE tag does not exists")]
    fn not_found() {
        let mut data = Cursor::new((1..200).collect::<Vec<u8>>());
        Meta::read(&mut data).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid APE version")]
    fn invalid_ape_version() {
        let mut data = Cursor::new(Vec::<u8>::new());
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(1000).unwrap();
        data.write_all(&[0; 20]).unwrap();
        Meta::read(&mut data).unwrap();
    }
}
