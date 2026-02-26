//! A Rust library for parsing and generating wobbly character-based waveforms (ASCII/UTF-8).
//! Converts `N` lines of ASCII representations of samples into callback calls 
//! for each character.
//!
//! ## Callback arguments
//! - `timestamp`: The time just before the character, in ticks. Each character represents 1 tick.
//! - `values`: An array of `N` boolean values, one for each line.
//! - `changed`: `true` if any of the lines changed since the last callback. If
//!      you're only interested in changes, ignore calls where `changed` is `false`.
//!
//! ## Character Mapping
//! - `'_'`: Represents `false`.
//! - `'|'`: Represents an edge. The value depends on the next character:
//!   - If followed by `'|'` or the end of the string: Toggle the last value.
//!   - If followed by `'_'`: Represents `false`.
//!   - Otherwise: Represents `true`.
//! - Any other character (e.g., `' '`, `'X'`, `'M'`): Represents `true`.
//!
//! ## Example
//! ```
//! use wobblechar::parser::lines_to_callback;
//! let lines = ["_| ", "|  "];
//! let expect = [(0, [false, true], true), (1, [true, true], true), (2, [true, true], false)];
//! let mut i = 0;
//! lines_to_callback(lines, 0, 1, |timestamp, values, changed| {
//!     assert_eq!(expect[i], (timestamp, values, changed), "Mismatch at index {}", i);
//!     i += 1;
//! });
//! ```
//!
//! # Specialized Version
//! For single-line cases, use `line_to_callback<F>()` instead.

// parse_waveform(text: &str) -> Vec<bool>
// generate_waveform(data: &[bool]) -> String
// generate_waveform_with_chars(data: &[bool], high_char: char, low_char: char) -> String (voor UTF-8 ondersteuning).

#![cfg_attr(not(feature = "std"), no_std)]  // Set no_std if std is off

// #[cfg(feature = "std")]
// extern crate std;

#[cfg(not(feature = "std"))]
extern crate alloc;  // For allocation-types (eq. Vec, String) in no_std


pub mod parser;
pub use parser::{builder::Builder, Item};
// pub use crate::parser::{line_to_callback, lines_to_callback};
