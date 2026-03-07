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
//! use wobblechar::Builder;
//! let expect = [(0, [false, true], false), (1, [true, false], true)];
//! for item in Builder::<2>::new_from_string("_‾\n‾_").with_def_bool_mapper().build() {
//!     let (exp_idx, exp_vals, exp_changed) = expect[item.index];
//!     assert_eq!(item.index, exp_idx);
//!     assert_eq!([item.values[0], item.values[1]], exp_vals);
//!     assert_eq!(item.changed, exp_changed);
//! }
//! ```

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
