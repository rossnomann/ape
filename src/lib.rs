//! A library for reading and writing APEv2 tags.
//!
//! An APE tag is a tag used to add metadata (title, artist, album, etc...) to digital audio files.
//!
//! Read the [specification][1] for more information.
//!
//! # Examples
//!
//! ## Creating a tag
//!
//! ```no_run
//! use ape::{Item, ItemType, Tag, write_to_path};
//!
//! let mut tag = Tag::new();
//! let item = Item::new("artist", ItemType::Text, "Artist Name").unwrap();
//! tag.set_item(item);
//! write_to_path(&tag, "path/to/file").unwrap();
//! ```
//!
//! ## Reading a tag
//!
//! ```no_run
//! use ape::read_from_path;
//!
//! let tag = read_from_path("path/to/file").unwrap();
//! let item = tag.item("artist").unwrap();
//! let value: &str = item.try_into().unwrap();
//! println!("{}", value);
//! ```
//!
//! ## Updating a tag
//!
//! ```no_run
//! use ape::{Item, ItemType, write_to_path, read_from_path};
//!
//! let path = "path/to/file";
//! let mut tag = read_from_path(path).unwrap();
//! let item = Item::new("album", ItemType::Text, "Album Name").unwrap();
//! tag.set_item(item);
//! tag.remove_items("cover");
//! write_to_path(&tag, path).unwrap();
//! ```
//!
//! ## Deleting a tag
//!
//! ```no_run
//! use ape::remove_from_path;
//!
//! remove_from_path("path/to/file").unwrap();
//! ```
//!
//! [1]: http://wiki.hydrogenaud.io/index.php?title=APEv2_specification
//!

#![warn(missing_docs)]

pub use self::{
    error::{Error, Result},
    item::{Item, ItemType},
    tag::{Tag, read_from, read_from_path, remove_from, remove_from_path, write_to, write_to_path},
};

mod error;
mod item;
mod meta;
mod tag;
mod util;
