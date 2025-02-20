use crate::{
    error::{Error, Result},
    util::{ID3V1_OFFSET, probe_ape, probe_id3v1, probe_lyrics3v2},
};
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Read, Seek, SeekFrom};

pub(super) const APE_VERSION: u32 = 2000;

#[derive(Debug)]
pub(super) struct Meta {
    // Tag size in bytes including footer and all tag items excluding the header.
    pub(super) size: u32,
    // Position of the metadata.
    pub(super) position: MetaPosition,
    // Tag contains a header.
    pub(super) has_header: bool,
    // Number of items in the Tag.
    pub(super) item_count: u32,
    // Initial position of the Tag items.
    pub(super) start_pos: u64,
    // End position of the Tag items.
    pub(super) end_pos: u64,
}

impl Meta {
    pub(super) fn read<R: Read + Seek>(reader: &mut R) -> Result<Meta> {
        const APE_HEADER_SIZE: i64 = 32;

        let mut found = probe_ape(reader, SeekFrom::End(-APE_HEADER_SIZE))? || probe_ape(reader, SeekFrom::Start(0))?;
        // When located at the end of an MP3 file, an APE tag should be placed after
        // the last frame, just before the ID3v1 tag (if any).
        if !found && probe_id3v1(reader)? {
            found = probe_ape(reader, SeekFrom::End(ID3V1_OFFSET - APE_HEADER_SIZE))?;
            if !found {
                // ID3v1 tag maybe preceded by Lyrics3v2: http://id3.org/Lyrics3v2
                let size = probe_lyrics3v2(reader)?;
                if size != -1 {
                    found = probe_ape(reader, SeekFrom::End(ID3V1_OFFSET - size - APE_HEADER_SIZE))?;
                }
            }
        }
        if !found {
            return Err(Error::TagNotFound);
        }
        if reader.read_u32::<LittleEndian>()? != APE_VERSION {
            return Err(Error::InvalidApeVersion);
        }
        let size = reader.read_u32::<LittleEndian>()?;
        let item_count = reader.read_u32::<LittleEndian>()?;
        let flags = MetaFlags::from_raw(reader.read_u32::<LittleEndian>()?);
        // The following 8 bytes are reserved
        const RESERVED_BYTES_NUM: i64 = 8;
        let end_pos = reader.seek(SeekFrom::Current(RESERVED_BYTES_NUM))?;
        Ok(Meta {
            size,
            position: flags.position,
            has_header: flags.has_header,
            item_count,
            start_pos: match flags.position {
                MetaPosition::Header => end_pos,
                MetaPosition::Footer => end_pos - size as u64,
            },
            end_pos: match flags.position {
                MetaPosition::Header => {
                    let mut pos = end_pos + size as u64;
                    if flags.has_footer {
                        pos -= APE_HEADER_SIZE as u64;
                    }
                    pos
                }
                MetaPosition::Footer => end_pos - APE_HEADER_SIZE as u64,
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) enum MetaPosition {
    // It's header of the tag.
    Header,
    // It's footer of the tag.
    Footer,
}

const HAS_HEADER: u32 = 1 << 31;
const HAS_NO_FOOTER: u32 = 1 << 30;
const IS_HEADER: u32 = 1 << 29;

struct MetaFlags {
    position: MetaPosition,
    has_header: bool,
    has_footer: bool,
}

impl MetaFlags {
    fn from_raw(raw: u32) -> Self {
        Self {
            position: if raw & IS_HEADER != 0 {
                MetaPosition::Header
            } else {
                MetaPosition::Footer
            },
            has_header: raw & HAS_HEADER != 0,
            has_footer: raw & HAS_NO_FOOTER == 0,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};
    use std::io::{Cursor, Write};

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
        assert_eq!(meta.position, MetaPosition::Footer);
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
        assert_eq!(meta.position, MetaPosition::Header);
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
        assert_eq!(meta.position, MetaPosition::Footer);
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
        assert_eq!(meta.position, MetaPosition::Footer);
        assert!(!meta.has_header);
        assert_eq!(562, meta.start_pos);
        assert_eq!(600, meta.end_pos);
    }

    #[test]
    fn not_found() {
        let mut data = Cursor::new((1..200).collect::<Vec<u8>>());
        let err = Meta::read(&mut data).unwrap_err().to_string();
        assert_eq!(err, "APE tag does not exist");
    }

    #[test]
    fn invalid_ape_version() {
        let mut data = Cursor::new(Vec::<u8>::new());
        data.write_all(b"APETAGEX").unwrap();
        data.write_u32::<LittleEndian>(1000).unwrap();
        data.write_all(&[0; 20]).unwrap();
        let err = Meta::read(&mut data).unwrap_err().to_string();
        assert_eq!(err, "invalid APE version");
    }
}
