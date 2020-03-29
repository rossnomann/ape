# RUST-APE

A library for reading and writing [APEv2 tags][1].

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/rossnomann/rust-ape/CI?style=flat-square)](https://github.com/rossnomann/rust-ape/actions/)
[![Downloads](https://img.shields.io/crates/d/ape.svg?style=flat-square)](https://crates.io/crates/ape/)
[![Documentation](https://img.shields.io/badge/docs-rs-yellowgreen.svg?style=flat-square)](https://docs.rs/ape)

## Changelog

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
