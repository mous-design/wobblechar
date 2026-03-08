use log::{error, warn};
pub mod mapper;
mod line_parser;
pub mod builder;
use core::marker::PhantomData;
use core::iter::Iterator;
use heapless::Vec;
use line_parser::{LineParser, LineIterator};

pub use mapper::{Entry, BoolMapper, NumMapper, LookupMap, Mapper};
pub use line_parser::ParseError;

/// A single timestep produced by the [`Parser`] iterator.
///
/// Each `Item` corresponds to one character position across all `N` lines.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Item<T,const N:usize > {
    /// The decoded value for each line at this timestep.
    pub values: Vec<T, N>,
    /// Zero-based index of this timestep (needed in case you filter out items).
    pub index: usize,
    /// `true` if any value changed compared to the previous timestep.
    /// Always `true` if any line contains a toggle character at this position, 
    ///   even if the resulting value is the same as before.
    /// Always `false` for the very first timestep (no previous to compare against).
    ///   Except when first char is a toggle, then it is `true` because the toggle forces a change.
    pub changed: bool,
}
impl <T, const N:usize> Item<T, N> {
    fn new() -> Self {
        let values: Vec<T, N> = Vec::new();
        Self {
            values,
            index: 0,
            changed: false,
        }
    }
}

type MapOut<M> = <<M as Mapper>::Map as LookupMap>::Out;

/// Iterator over decoded timesteps for `N` signal lines. 
/// Don't be confused: N is NOT the number of input lines, since some lines are 
/// ignored (empty, only comment) and others are joined (same label). So the N
/// is actually the number of output lines, or, to put it differently, the 
/// number of signal lines you are here simulating.
///
/// Yields one [`Item`] per character position. Typically constructed via
/// [`Builder`](crate::Builder) rather than directly.
pub struct Parser<'a, const N:usize, M>
    where M:Mapper
{
    line_iters: Vec<LineIterator<'a>,N>,
    index: usize,
    last_values: Option<Vec<MapOut<M>,N>>,
    mapper: M,
    _phantom: PhantomData<MapOut<M>>,
}

impl<'a, M, const N: usize> Parser<'a, N, M>
    where M: Mapper,
{
    pub fn new(string: &'a str, mapper: M) -> Result<Self, ParseError> {
        let lines = LineParser::new(string).get_lines::<N>()?;
        let line_iters: Vec<LineIterator<'a>, N> = lines
            .into_iter()
            .map(|line| line.into_iter())
            .collect();
        let last_values: Option<Vec<MapOut<M>, N>> = None;
        Ok(Self {
            line_iters,
            index: 0,
            last_values,
            mapper,
            _phantom: PhantomData,
        })
    }
}

impl<const N:usize, M> Iterator for Parser<'_, N, M>
    where M: Mapper
{
    type Item = Item<MapOut<M>, N>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut item = Item::<MapOut<M>,N>::new();
        let mut eols:Vec<usize,N> = Vec::new();
        for (line_idx,iter) in self.line_iters.iter_mut().enumerate() {
            let last_value = self.last_values.as_ref().map(|last_values| last_values[line_idx]);
            let c = iter.next();
            let val = if let Some(c) = c {
                if self.mapper.is_toggle(c) {
                    // Toggle character forces changed to true
                    item.changed = true;
                    if let Some(last_value) = last_value {
                        // toggle the last value
                        self.mapper.toggle(last_value)
                    } else {
                        // If no last value, Look at next character
                        if let Some(n) = iter.peek() {
                            if self.mapper.is_toggle(n) {
                                // This is an unexpected case: There is no 
                                //last-value AND no ext value. Assume low at start.
                                warn!("Cannot infer level for toggle at index {}, using default low as start.", self.index);
                                self.mapper.high()
                            } else if let Some(v) = self.mapper.value(n) {
                                v
                            } else {
                                error!("Unknown character, at index {}, stop parsing", self.index);
                                return None;
                            }
                        } else {
                            // This is an unexpected case: There is no last-value AND no no ext value.
                            warn!("Cannot infer level for toggle at index {}, using default low as start.", self.index);
                            self.mapper.high()
                        }
                    }
                } else if let Some(v) = self.mapper.value(c) {
                    v
                } else {
                    error!("Unknown character, at index {}, stop parsing", self.index);
                    return None;
                }
            } else {
                // End of line here.
                let _ = eols.push(line_idx);
                // Keep the last value, or the default if none.
                last_value.unwrap_or_else(|| self.mapper.default())
            };
            // If not yet changed, IF there is a last value, compare. If not, don't mark as changed
            item.changed = item.changed || if let Some(last_value) = last_value {
                val != last_value
            } else {
                false
            };
            let _ = item.values.push(val);
        }
        item.index = self.index;
        self.index += 1;
        self.last_values = Some(item.values.clone());

        // If any line returned some value, return the item. 
        if eols.len() < self.line_iters.len() {
            if !eols.is_empty() {
                warn!("Line(s) {:?} are only {} long. Other lines are longer.", eols, self.index);
            }
            Some(item)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::builder::Builder;

    use super::*;

    #[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
    struct Rec {
        changed: bool,
        value: bool,
    }

    struct AsciiToEvtTc {
        case: &'static str,
        exp: &'static [Rec],
    }

    const ASCII_TO_EVT_CASES: [AsciiToEvtTc; 22] = [
        // # Special cases
        // Empty
        AsciiToEvtTc { case: "", exp: &[] },
        // No edge low
        AsciiToEvtTc { case: "_", exp: &[Rec {changed: false, value:false}] },
        // No edge high
        AsciiToEvtTc { case: "‾", exp: &[Rec {changed: false, value:true}] },
        // # Cases with |
        // Only |
        AsciiToEvtTc { case: "|", exp: &[Rec {changed: true, value:true}] },
        // Only | end after _
        AsciiToEvtTc { case: "_|", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
        ]},
        // Only | end after X
        AsciiToEvtTc { case: "‾|", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
        ]},
        // Only || -> Assumes low at start
        AsciiToEvtTc { case: "||", exp: &[
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
        ]},
        // || at start -> Assumes low at start
        AsciiToEvtTc { case: "||_", exp: &[
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
            Rec {changed: false, value:false},
        ]},
        // Normal edge up
        AsciiToEvtTc { case: "_|‾", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: false, value:true},
        ]},
        // Normal edge down
        AsciiToEvtTc { case: "‾|_", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: false, value:false},
        ]},
        // Only | between low
        AsciiToEvtTc { case: "_|_", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
        ]},
        // Only | between high
        AsciiToEvtTc { case: "‾|‾", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
        ]},
        // Double | between low
        AsciiToEvtTc { case: "_||_", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
            Rec {changed: false, value:false},
        ]},
        // Double | between high
        AsciiToEvtTc { case: "‾||‾", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
            Rec {changed: false, value:true},
        ]},

        // Normal pulise from low
        AsciiToEvtTc { case: "_|‾|_", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: false, value:false},
        ]},
        // Normal pulse from high
        AsciiToEvtTc { case: "‾|_|‾", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: false, value:true},
        ]},
        // Triple case from low
        AsciiToEvtTc { case: "_|||_", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
        ]},
        // Triple case from high
        AsciiToEvtTc { case: "‾|||‾", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
        ]},

        // # Cases without | and other high char
        // Pulse between low
        AsciiToEvtTc { case: "_X_", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
            Rec {changed: true, value:false},
        ]},
        // Pulse between high
        AsciiToEvtTc { case: "X_X", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
            Rec {changed: true, value:true},
        ]},
        // Only | end after low
        AsciiToEvtTc { case: "_X", exp: &[
            Rec {changed: false, value:false},
            Rec {changed: true, value:true},
        ]},
        // Only | end after high
        AsciiToEvtTc { case: "X_", exp: &[
            Rec {changed: false, value:true},
            Rec {changed: true, value:false},
        ]},
    ];

    #[test]
    fn test_parse() {
        let cases = &ASCII_TO_EVT_CASES;
        for tc in cases {
            let samples: Vec<_,5> = 
                Builder::<1>::new_from_string(tc.case).with_def_bool_mapper().build().
                map(|item| 
                    (item.index, Rec { changed: item.changed, value: item.values[0] })
                ).collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index,sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct RecMulti<const N: usize> {
        changed: bool,
        values: [bool; N],
    }
    impl<const N: usize> Default for RecMulti<N> {
        fn default() -> Self {
            Self {
                changed: false,
                values: [false; N],
            }
        }
    }

    #[derive(Copy, Clone)]
    struct AsciisToEvtTc<const N: usize> {
        case: &'static str,
        exp: &'static [RecMulti<N>],
    }
    const ASCIIS_TO_EVT_CASES: [AsciisToEvtTc<2>; 6] = [
        // Special cases
        AsciisToEvtTc { case: "\n", exp: &[] },

        AsciisToEvtTc { case: "_\n‾", exp: &[
            RecMulti { changed: false, values: [false, true]},
        ]},

        // Different lengths
        AsciisToEvtTc { case: "_\n‾‾", exp: &[
            RecMulti { changed: false, values: [false, true]},
            RecMulti { changed: false, values: [false, true]},
        ]},
        AsciisToEvtTc { case: "__\n‾", exp: &[
            RecMulti { changed: false, values: [false, true]},
            RecMulti { changed: false, values: [false, true]},
        ]},

        // Normal cases
        AsciisToEvtTc {
            case: 
                "___‾‾_
                 _‾____",
            exp: &[
                RecMulti { changed: false, values: [false, false]},
                RecMulti { changed: true, values: [false, true]},
                RecMulti { changed: true, values: [false, false]},
                RecMulti { changed: true, values: [true, false]},
                RecMulti { changed: false, values: [true, false]},
                RecMulti { changed: true, values: [false, false]},
            ]},
        AsciisToEvtTc {
            case: 
                "‾‾‾__‾
                 ‾_‾‾‾‾",
            exp: &[
                RecMulti { changed: false, values: [true, true]},
                RecMulti { changed: true, values: [true, false]},
                RecMulti { changed: true, values: [true, true]},
                RecMulti { changed: true, values: [false, true]},
                RecMulti { changed: false, values: [false, true]},
                RecMulti { changed: true, values: [true, true]},
            ]},        
    ];

    #[test]
    fn test_parse_multi() {
        let cases = &ASCIIS_TO_EVT_CASES;
        for tc in cases {
            let samples: Vec<_,6> = 
                Builder::<2>::new_from_string(tc.case).with_def_bool_mapper().build().
                map(|item:Item<bool,_>| 
                    (item.index, RecMulti { changed: item.changed, values: [item.values[0], item.values[1]] })
                ).collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index,sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }

        // Now test labels
    const ASCIIS_TO_EVT_CASES_LABS: [AsciisToEvtTc<3>; 1] = [
        AsciisToEvtTc { case: "
            L1:_‾ # First line with label 1
            L2:‾‾ # First line with label 2
            _‾_‾  # No label
            L1:‾_ # Label 1 continued
            L2:_‾ # Label 2 continued
        ", exp: &[
            RecMulti { changed: false, values: [false, true, false]},
            RecMulti { changed: true, values: [true, true, true]},
            RecMulti { changed: true, values: [true, false, false]},
            RecMulti { changed: true, values: [false, true, true]},
        ]},
    ];

   #[test]
    fn test_parse_multi_with_labels() {
        let cases = &ASCIIS_TO_EVT_CASES_LABS;
        for tc in cases {
            let samples: Vec<_,6> = 
                Builder::<3>::new_from_string(tc.case).with_def_bool_mapper().build().
                map(|item:Item<bool,_>| 
                    (item.index, RecMulti { changed: item.changed, values: [item.values[0], item.values[1], item.values[2]] })
                ).collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index,sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }
    // Tests for different mappers
    use super::mapper::Entry;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    struct RecNum<T> {
        changed: bool,
        value: T,
    }
    struct DefNumTc {
        case: &'static str,
        exp: &'static [RecNum<u8>],
    }

    const DEF_NUM_CASES: [DefNumTc; 5] = [
        DefNumTc { case: "_", exp: &[RecNum { changed: false, value: 0 }]},
        DefNumTc { case: "‾", exp: &[RecNum { changed: false, value: 1 }]},
        DefNumTc { case: "_|‾", exp: &[
            RecNum { changed: false, value: 0 },
            RecNum { changed: true,  value: 1 },
            RecNum { changed: false, value: 1 },
        ]},

        DefNumTc { case: "‾|_", exp: &[
            RecNum { changed: false, value: 1 },
            RecNum { changed: true,  value: 0 },
            RecNum { changed: false, value: 0 },
        ]},

        DefNumTc { case: "_|‾|_", exp: &[
            RecNum { changed: false, value: 0 },
            RecNum { changed: true,  value: 1 },
            RecNum { changed: false, value: 1 },
            RecNum { changed: true,  value: 0 },
            RecNum { changed: false, value: 0 },
        ]},
    ];

    #[test]
    fn test_parse_def_num_mapper() {
        for tc in &DEF_NUM_CASES {
            let samples: Vec<_, 5> =
                Builder::<1>::new_from_string(tc.case).with_def_num_mapper::<u8>().build()
                    .map(|item| (item.index, RecNum { changed: item.changed, value: item.values[0] }))
                    .collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index, sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }

    // --- Const Bool Mapper ---
    // Custom chars: 'H' = high (true), 'L' = low (false), '|' = toggle
    const CONST_BOOL_MAP: [(char, Entry<bool>); 3] = [
        ('H', Entry::Value(true)),
        ('L', Entry::Value(false)),
        ('|', Entry::Toggle),
    ];
    struct ConstBoolTc {
        case: &'static str,
        exp: &'static [Rec],
    }

    const CONST_BOOL_CASES: [ConstBoolTc; 6] = [
        ConstBoolTc { case: "L", exp: &[Rec { changed: false, value: false }]},
        ConstBoolTc { case: "H",exp: &[Rec { changed: false, value: true }]},
        ConstBoolTc { case: "LH",exp: &[
            Rec { changed: false, value: false },
            Rec { changed: true,  value: true },
        ]},
        ConstBoolTc { case: "L|H",exp: &[
            Rec { changed: false, value: false },
            Rec { changed: true,  value: true },
            Rec { changed: false, value: true },
        ]},
        ConstBoolTc { case: "H|L",exp: &[
            Rec { changed: false, value: true },
            Rec { changed: true,  value: false },
            Rec { changed: false, value: false },
        ]},
        ConstBoolTc { case: "L|H|L",exp: &[
            Rec { changed: false, value: false },
            Rec { changed: true,  value: true },
            Rec { changed: false, value: true },
            Rec { changed: true,  value: false },
            Rec { changed: false, value: false },
        ]},
    ];

    #[test]
    fn test_parse_const_bool_mapper() {
        for tc in &CONST_BOOL_CASES {
            let samples: Vec<_, 5> =
                Builder::<1>::new_from_string(tc.case).with_const_bool_map(&CONST_BOOL_MAP).build()
                    .map(|item| (item.index, Rec { changed: item.changed, value: item.values[0] }))
                    .collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index, sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }

    // --- Const Num Mapper ---
    // Custom chars: 'X' = 5 (high), '_' = 0 (low), '|' = toggle
    // NumMapper toggle: toggle(v) = min + max - v = 0 + 5 - v
    const CONST_NUM_MAP_I8: [(char, Entry<i8>); 3] = [
        ('X', Entry::Value(5)),
        ('_', Entry::Value(0)),
        ('|', Entry::Toggle),
    ];

    struct ConstNumTc {
        case: &'static str,
        exp: &'static [RecNum<i8>],
    }

    const CONST_NUM_CASES: [ConstNumTc; 6] = [

        ConstNumTc { case: "_", exp: &[RecNum { changed: false, value: 0 }]},
        ConstNumTc { case: "X", exp: &[RecNum { changed: false, value: 5 }]},
        ConstNumTc { case: "_X_", exp: &[
            RecNum { changed: false, value: 0 },
            RecNum { changed: true,  value: 5 },
            RecNum { changed: true,  value: 0 },
        ]},
        ConstNumTc { case: "_|X", exp: &[
            RecNum { changed: false, value: 0 },
            RecNum { changed: true,  value: 5 },
            RecNum { changed: false, value: 5 },
        ]},
        ConstNumTc { case: "X|_", exp: &[
            RecNum { changed: false, value: 5 },
            RecNum { changed: true,  value: 0 },
            RecNum { changed: false, value: 0 },
        ]},
        ConstNumTc { case: "_|X|_", exp: &[
            RecNum { changed: false, value: 0 },
            RecNum { changed: true,  value: 5 },
            RecNum { changed: false, value: 5 },
            RecNum { changed: true,  value: 0 },
            RecNum { changed: false, value: 0 },
        ]},
    ];

    #[test]
    fn test_parse_const_num_mapper() {
        for tc in &CONST_NUM_CASES {
            let samples: Vec<_, 5> =
                Builder::<1>::new_from_string(tc.case).with_const_num_mapper::<i8>(&CONST_NUM_MAP_I8).build()
                    .map(|item| (item.index, RecNum { changed: item.changed, value: item.values[0] }))
                    .collect();
            assert_eq!(tc.exp.len(), samples.len(), "unexpected number of values for case '{}'", tc.case);
            for (i, (index, sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, tc.case);
                assert_eq!(tc.exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, tc.case);
            }
        }
    }

    // --- Hash Bool Mapper (requires std) ---
    #[cfg(feature = "std")]
    #[test]
    fn test_parse_hash_bool_mapper() {
        use std::collections::HashMap;
        let map: HashMap<char, Entry<bool>> = [
            ('H', Entry::Value(true)),
            ('L', Entry::Value(false)),
            ('|', Entry::Toggle),
        ].into();
        let cases: [(&str, &[Rec]); 4] = [
            ("L",    &[Rec { changed: false, value: false }]),
            ("H",    &[Rec { changed: false, value: true }]),
            ("L|H",  &[Rec { changed: false, value: false }, Rec { changed: true, value: true }, Rec { changed: false, value: true }]),
            ("H|L",  &[Rec { changed: false, value: true  }, Rec { changed: true, value: false }, Rec { changed: false, value: false }]),
        ];
        for (case, exp) in &cases {
            let samples: Vec<_, 5> =
                Builder::<1>::new_from_string(case).with_hash_bool_mapper(&map).build()
                    .map(|item| (item.index, Rec { changed: item.changed, value: item.values[0] }))
                    .collect();
            assert_eq!(exp.len(), samples.len(), "unexpected number of values for case '{}'", case);
            for (i, (index, sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, case);
                assert_eq!(exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, case);
            }
        }
    }

    // --- Hash Num Mapper (requires std) ---
    #[cfg(feature = "std")]
    #[test]
    fn test_parse_hash_num_mapper() {
        use std::collections::HashMap;
        let map: HashMap<char, Entry<i8>> = [
            ('X', Entry::Value(5i8)),
            ('_', Entry::Value(0i8)),
            ('|', Entry::Toggle),
        ].into();
        let cases: [(&str, &[RecNum<i8>]); 4] = [
            ("_",    &[RecNum { changed: false, value: 0 }]),
            ("X",    &[RecNum { changed: false, value: 5 }]),
            ("_|X",  &[RecNum { changed: false, value: 0 }, RecNum { changed: true, value: 5 }, RecNum { changed: false, value: 5 }]),
            ("X|_",  &[RecNum { changed: false, value: 5 }, RecNum { changed: true, value: 0 }, RecNum { changed: false, value: 0 }]),
        ];
        for (case, exp) in &cases {
            let samples: Vec<_, 5> =
                Builder::<1>::new_from_string(case).with_hash_num_mapper::<i8>(&map).build()
                    .map(|item| (item.index, RecNum { changed: item.changed, value: item.values[0] }))
                    .collect();
            assert_eq!(exp.len(), samples.len(), "unexpected number of values for case '{}'", case);
            for (i, (index, sample)) in samples.iter().enumerate() {
                assert_eq!(i, *index, "index mismatch at index {} for case '{}'", i, case);
                assert_eq!(exp[i], *sample, "Sample mismatch at index {} for case '{}'", i, case);
            }
        }
    }
    
}