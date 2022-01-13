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
//! use ape::{Item, Tag, write_to_path};
//!
//! let mut tag = Tag::new();
//! let item = Item::from_text("artist", "Artist Name").unwrap();
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
//! println!("{:?}", item.value);
//! ```
//!
//! ## Updating a tag
//!
//! ```no_run
//! use ape::{Item, write_to_path, read_from_path};
//!
//! let path = "path/to/file";
//! let mut tag = read_from_path(path).unwrap();
//! let item = Item::from_text("album", "Album Name").unwrap();
//! tag.set_item(item);
//! tag.remove_item("cover");
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
    item::{Item, ItemValue},
    tag::{read_from, read_from_path, remove_from, remove_from_path, write_to, write_to_path, Tag},
};

mod error;
mod item;
mod meta;
mod tag;
mod util;
