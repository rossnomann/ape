# APE

A library for reading and writing [APEv2 tags][1].

[![CI Status](https://img.shields.io/github/actions/workflow/status/rossnomann/ape/ci.yml?style=flat-square)](https://github.com/rossnomann/ape/actions/)
[![Downloads](https://img.shields.io/crates/d/ape.svg?style=flat-square)](https://crates.io/crates/ape/)
[![Documentation](https://img.shields.io/badge/docs-rs-yellowgreen.svg?style=flat-square)](https://docs.rs/ape)

## Changelog

### 0.6.0 (04.02.2025)

Fixed multiple values support.
You may need to overwrite your tags if you have called the `Tag::add_item` method with the same key multiple times,
since the spec [states](https://wiki.hydrogenaud.io/index.php?title=APE_Tag_Item) that
"Every Tag Item Key can only occures (at most) once".

- Updated byteorder to 1.5.
- Added `TryFrom<&Item>` trait implementation for `Vec<&str>` and `&str`.
- Added `TryFrom<Item>` trait implementation for `Vec<String>` and `String`.
- Removed `ItemValue` enum and `Item.value` field; use `TryFrom` instead.
- Added `ItemType { Binary, Locator, Text }` enum.
- Added `Item::new(key, type, value)` method.
- Removed `Item::from_binary`, `Item::from_locator` and `Item::from_text` methods; use `Item::new` method instead.
- Added `Item.add_value` method.
- Removed `Tag::add_item` method; use `Item.add_value` instead.
- Added `Item::with_type` method.
- Added `Item::with_value` method.
- Added `Item.get_type` method.
- Changed `Error` enum:
  - Removed: `FromUtf8`, `ParseInt`
  - Added: `ParseItemKey`, `ParseItemBinary`, `ParseItemValue`, `ParseLyrics3V2SizeStr`, `ParseLyrics3V2SizeInt`.
  - Changed:
    - `BadItemType` -> `InvalidItemType(u32)`.
    - `BadTagSize` -> `InvalidTagSize`.

### 0.5.0 (11.01.2023)

- Added support for multiple values under same key:
  - Added `Tag::items` method.
  - Added `Tag::add_item` method.
  - Replaced `Tag::remove_item` by `Tag::remove_items` method.
  `Tag::item` method returns a first found item.
  `Tag::set_item` removes all items under the given key and adds a new one.
- Added derive `Clone` for `Item` and `ItemValue` structs.

### 0.4.0 (13.01.2022)

- Switched to 2021 edition.
- Updated byteorder to 1.4
- Support reading/writing/removing tags from opened files.
- Case-insensitive key comparison.
- Support writing an empty tag.

### 0.3.0 (29.03.2020)

- Switched to 2018 edition.
- Updated byteorder to 1.3
- Fixed type parameters in `Item::from_locator` and `Item::from_text`.
- Removed use of deprecated `Error::description`.
- Lowercase error description.
- `Item::to_vec` method is private now.
- Removed `items` field from the `Tag` struct.
- Added `Tag::iter()` method.
- Added `IntoIterator` implementation for `Tag` struct.
- `Tag::write` method replaced by `write` function.

### 0.2.0 (10.12.2017)

- Use byteorder 1.0.0.

### 0.1.2 (18.05.2016)

- Small internal improvements.

### 0.1.1 (21.01.2016)

- Use `Result<()>` instead of `Option<Error>`.

### 0.1.0 (16.01.2016)

- First release.

## LICENSE

The MIT License (MIT)

[1]: http://wiki.hydrogenaud.io/index.php?title=APEv2_specification
