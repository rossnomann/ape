//! A library for reading and writing APEv2 tags.
//!
//! An APE tag is a tag used to add metadata (title, artist, album, etc...) to digital audio files.
//!
//! Read the [specification][1] for more information.
//! [1]: http://wiki.hydrogenaud.io/index.php?title=APEv2_specification
//!
//! # Examples
//!
//! ## Creating a tag
//!
//! ```no_run
//! use ape::{write, Item, Tag};
//!
//! let mut tag = Tag::new();
//! let item = Item::from_text("artist", "Artist Name").unwrap();
//! tag.set_item(item);
//! write(&tag, "path/to/file").unwrap();
//! ```
//!
//! ## Reading a tag
//!
//! ```no_run
//! use ape::read;
//!
//! let tag = read("path/to/file").unwrap();
//! let item = tag.item("artist").unwrap();
//! println!("{:?}", item.value);
//! ```
//!
//! ## Updating a tag
//!
//! ```no_run
//! use ape::{read, write, Item};
//!
//! let path = "path/to/file";
//! let mut tag = read(path).unwrap();
//! let item = Item::from_text("album", "Album Name").unwrap();
//! tag.set_item(item);
//! tag.remove_item("cover");
//! write(&tag, path).unwrap();
//! ```
//!
//! ## Deleting a tag
//!
//! ```no_run
//! use ape::remove;
//!
//! remove("path/to/file").unwrap();
//! ```
//!

#![warn(missing_docs)]

pub use self::{
    error::{Error, Result},
    item::{Item, ItemValue},
    tag::{read, remove, write, Tag},
};

mod error;
mod item;
mod meta;
mod tag;
mod util;
