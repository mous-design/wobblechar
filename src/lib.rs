//! Parse ASCII/UTF-8 waveform diagrams into a stream of decoded samples.
//!
//! `wobblechar` creates a bridge between the human-friendly world of text
//! waveforms and the structured data needed for programmatic processing. It was
//! born out of the need to make testing less tedious and more intuitive. 
//! 
//! `wobblechar` takes one or more lines of text waveforms and converts them 
//! into an iterator of [`Item`]s, one per character position. Each item carries
//! the decoded value for every line and a `changed` flag that is `true` 
//! whenever at least one line changed since the previous timestep.
//!
//! ## Getting started
//!
//! Use [`Builder`] to construct a parser:
//!
//! ```
//! use wobblechar::Builder;
//!
//! // Two-line waveform; each character is one timestep.
//! for item in Builder::<2>::new_from_string("
//! _|‾|_
//! ‾|_|‾
//! ")
//!     .with_def_bool_mapper()
//!     .build()
//! {
//!     println!("t={} values={:?} changed={}", item.index, item.values, item.changed);
//! }
//! ```
//!
//! ## Default character mapping
//!
//! The default mappers (`with_def_bool_mapper` / `with_def_num_mapper`) use:
//!
//! | Character | Meaning |
//! |-----------|---------|
//! | `_`       | Low (`false` / `0`) |
//! | `\|`      | Edge — toggles the previous value |
//! | anything else (e.g. `‾`, `X`) | High (`true` / `1`) |
//!
//! Custom mappings can be provided via [`Builder::with_const_bool_map`],
//! [`Builder::with_const_num_mapper`], or (with the `std` feature) the
//! `HashMap`-based variants.
//!
//! ## Multi-line input format
//!
//! Lines are separated by `\n`. Leading/trailing whitespace and `#` comments
//! are stripped. Lines may carry a label prefix (`Name: content`) to allow
//! interleaved or non-contiguous line blocks:
//!
//! ```text
//! CLK: _|‾|_|‾|_   # clock
//! DAT: ___|‾‾‾|_   # data
//! # some comment
//! CLK: ‾‾‾___‾‾‾   # clock continued
//! DAT: ___|‾‾‾|__  # data continued
//! ```
//! 
//! Any set of characters, starting from the first non-whitespace character 
//! up to the first `:` is considered the label. Except of course for the
//! `#`-character.
//! 
//! ## `no_std` support
//!
//! The crate is `no_std` by default, which means you can use it in no_std 
//! environments without problems. If you have a project with std, enable the 
//! `std` feature for `HashMap`-backed mappers.


#![cfg_attr(not(feature = "std"), no_std)]  // Set no_std if std is off

pub mod parser;
pub use parser::{
    builder::Builder,
    Item,
    Parser,
    Entry,
    ParseError,
    BoolMapper,
    NumMapper,
    LookupMap,
    Mapper,
    mapper,
};
