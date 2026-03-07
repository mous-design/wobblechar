use log::{error, warn};
mod mapper;
mod line_parser;
pub mod builder;
use core::marker::PhantomData;
use core::iter::Iterator;
use mapper::{Mapper,BoolMapper, NumMapper, LookupMap};
use heapless::Vec;
use line_parser::{LineParser, LineIterator, ParseError};

pub struct Item<T,const N:usize > {
    pub values: Vec<T, N>,
    pub index: usize,
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
            let last_value = if let Some(ref last_values) = self.last_values {
                Some(last_values[line_idx])
            } else {
                None
            };
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
                            } else {
                                if let Some(v) = self.mapper.value(n) {
                                    v
                                } else {
                                    error!("Unknown character, at index {}, stop parsing", self.index);
                                    return None;
                                }
                            }
                        } else {
                            // This is an unexpected case: There is no last-value AND no no ext value.
                            warn!("Cannot infer level for toggle at index {}, using default low as start.", self.index);
                            self.mapper.high()
                        }
                    }
                } else {
                    if let Some(v) = self.mapper.value(c) {
                        v
                    } else {
                        error!("Unknown character, at index {}, stop parsing", self.index);
                        return None;
                    }
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
            if eols.len() > 0 {
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

    // # Special cases
    // Empty
    const CASE_1_1: &str = "";
    const EXP_1_1: [Rec; 0] = [];
    // No edge low
    const CASE_1_2: &str = "_";
    const EXP_1_2: [Rec; 1] = [Rec {changed: false, value:false}];
    // No edge high
    const CASE_1_3: &str = "‾";
    const EXP_1_3: [Rec; 1] = [Rec {changed: false, value:true}];

    // # Cases with |
    // Only |
    const CASE_1_4: &str = "|";
    const EXP_1_4: [Rec; 1] = [Rec {changed: true, value:true}];
    // Only | end after _
    const CASE_1_5: &str = "_|";
    const EXP_1_5: [Rec; 2] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
    ];
    // Only | end after X
    const CASE_1_6: &str = "‾|";
    const EXP_1_6: [Rec; 2] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
    ];
    // Only || -> Assumes low at start
    const CASE_1_7: &str = "||";
    const EXP_1_7: [Rec; 2] = [
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
    ];
    // || at start -> Assumes low at start
    const CASE_1_8: &str = "||_";
    const EXP_1_8: [Rec; 3] = [
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
        Rec {changed: false, value:false},
    ];
    // Normal edge up
    const CASE_1_9: &str = "_|‾";
    const EXP_1_9: [Rec; 3] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: false, value:true},
    ];
    // Normal edge down
    const CASE_1_10: &str = "‾|_";
    const EXP_1_10: [Rec; 3] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: false, value:false},
    ];
    // Only | between low
    const CASE_1_11: &str = "_|_";
    const EXP_1_11: [Rec; 3] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
    ];
    // Only | between high
    const CASE_1_12: &str = "‾|‾";
    const EXP_1_12: [Rec; 3] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
    ];
    // Double | between low
    const CASE_1_13: &str = "_||_";
    const EXP_1_13: [Rec; 4] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
        Rec {changed: false, value:false},
    ];
    // Double | between high
    const CASE_1_14: &str = "‾||‾";
    const EXP_1_14: [Rec; 4] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
        Rec {changed: false, value:true},
    ];

    // Normal pulise from low
    const CASE_1_15: &str = "_|‾|_";
    const EXP_1_15: [Rec; 5] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: false, value:false},
    ];
    // Normal pulse from high
    const CASE_1_16: &str = "‾|_|‾";
    const EXP_1_16: [Rec; 5] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: false, value:true},
    ];
    // Triple case from low
    const CASE_1_17: &str = "_|||_";
    const EXP_1_17: [Rec; 5] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
    ];
    // Triple case from high
    const CASE_1_18: &str = "‾|||‾";
    const EXP_1_18: [Rec; 5] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
    ];

    // # Cases without | and other high char
    // Pulse between low
    const CASE_2_1: &str = "_X_";
    const EXP_2_1: [Rec; 3] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
        Rec {changed: true, value:false},
    ];
    // Pulse between high
    const CASE_2_2: &str = "X_X";
    const EXP_2_2: [Rec; 3] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
        Rec {changed: true, value:true},
    ];
    // Only | end after low
    const CASE_2_3: &str = "_X";
    const EXP_2_3: [Rec; 2] = [
        Rec {changed: false, value:false},
        Rec {changed: true, value:true},
    ];
    // Only | end after high
    const CASE_2_4: &str = "X_";
    const EXP_2_4: [Rec; 2] = [
        Rec {changed: false, value:true},
        Rec {changed: true, value:false},
    ];

    const ASCII_TO_EVT_CASES: [AsciiToEvtTc; 22] = [
        AsciiToEvtTc { case: &CASE_1_1, exp: &EXP_1_1 },
        AsciiToEvtTc { case: &CASE_1_2, exp: &EXP_1_2 },
        AsciiToEvtTc { case: &CASE_1_3, exp: &EXP_1_3 },
        AsciiToEvtTc { case: &CASE_1_4, exp: &EXP_1_4 },
        AsciiToEvtTc { case: &CASE_1_5, exp: &EXP_1_5 },
        AsciiToEvtTc { case: &CASE_1_6, exp: &EXP_1_6 },
        AsciiToEvtTc { case: &CASE_1_7, exp: &EXP_1_7 },
        AsciiToEvtTc { case: &CASE_1_8, exp: &EXP_1_8 },
        AsciiToEvtTc { case: &CASE_1_9, exp: &EXP_1_9 },
        AsciiToEvtTc { case: &CASE_1_10, exp: &EXP_1_10 },
        AsciiToEvtTc { case: &CASE_1_11, exp: &EXP_1_11 },
        AsciiToEvtTc { case: &CASE_1_12, exp: &EXP_1_12 },
        AsciiToEvtTc { case: &CASE_1_13, exp: &EXP_1_13 },
        AsciiToEvtTc { case: &CASE_1_14, exp: &EXP_1_14 },
        AsciiToEvtTc { case: &CASE_1_15, exp: &EXP_1_15 },
        AsciiToEvtTc { case: &CASE_1_16, exp: &EXP_1_16 },
        AsciiToEvtTc { case: &CASE_1_17, exp: &EXP_1_17 },
        AsciiToEvtTc { case: &CASE_1_18, exp: &EXP_1_18 },
        AsciiToEvtTc { case: &CASE_2_1, exp: &EXP_2_1 },
        AsciiToEvtTc { case: &CASE_2_2, exp: &EXP_2_2 },
        AsciiToEvtTc { case: &CASE_2_3, exp: &EXP_2_3 },
        AsciiToEvtTc { case: &CASE_2_4, exp: &EXP_2_4 },
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

    // Special cases
    const CASE_3_1: &str = "\n";
    const EXP_3_1: [RecMulti<2>; 0] = [];

    const CASE_3_2: &str = "_\n‾";
    const EXP_3_2: [RecMulti<2>; 1] = [
        RecMulti { changed: false, values: [false, true]},
    ];

    // Different lengths
    const CASE_3_3: &str = "_\n‾‾";
    const EXP_3_3: [RecMulti<2>; 2] = [
        RecMulti { changed: false, values: [false, true]},
        RecMulti { changed: false, values: [false, true]},
    ];
    const CASE_3_4: &str = "__\n‾";
    const EXP_3_4: [RecMulti<2>; 2] = [
        RecMulti { changed: false, values: [false, true]},
        RecMulti { changed: false, values: [false, true]},
    ];

    // Normal cases
    const CASE_3_5: &str = 
        "___‾‾_
         _‾____";
    const EXP_3_5: [RecMulti<2>; 6] = [
        RecMulti { changed: false, values: [false, false]},
        RecMulti { changed: true, values: [false, true]},
        RecMulti { changed: true, values: [false, false]},
        RecMulti { changed: true, values: [true, false]},
        RecMulti { changed: false, values: [true, false]},
        RecMulti { changed: true, values: [false, false]},
    ];
    const CASE_3_6: &str =
        "‾‾‾__‾
         ‾_‾‾‾‾";
    const EXP_3_6: [RecMulti<2>; 6] = [
        RecMulti { changed: false, values: [true, true]},
        RecMulti { changed: true, values: [true, false]},
        RecMulti { changed: true, values: [true, true]},
        RecMulti { changed: true, values: [false, true]},
        RecMulti { changed: false, values: [false, true]},
        RecMulti { changed: true, values: [true, true]},
    ];

    const ASCIIS_TO_EVT_CASES: [AsciisToEvtTc<2>; 6] = [
        AsciisToEvtTc { case: &CASE_3_1, exp: &EXP_3_1 },
        AsciisToEvtTc { case: &CASE_3_2, exp: &EXP_3_2 },
        AsciisToEvtTc { case: &CASE_3_3, exp: &EXP_3_3 },
        AsciisToEvtTc { case: &CASE_3_4, exp: &EXP_3_4 },
        AsciisToEvtTc { case: &CASE_3_5, exp: &EXP_3_5 },
        AsciisToEvtTc { case: &CASE_3_6, exp: &EXP_3_6 },
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
    const CASE_4_1: &str = "
    L1:_‾ # First line with label 1
    L2:‾‾ # First line with label 2
    _‾_‾  # No label
    L1:‾_ # Label 1 continued
    L2:_‾ # Label 2 continued
    ";

    const EXP_4_1: [RecMulti<3>; 4] = [
        RecMulti { changed: false, values: [false, true, false]},
        RecMulti { changed: true, values: [true, true, true]},
        RecMulti { changed: true, values: [true, false, false]},
        RecMulti { changed: true, values: [false, true, true]},
    ];


    const ASCIIS_TO_EVT_CASES_LABS: [AsciisToEvtTc<3>; 1] = [
        AsciisToEvtTc { case: &CASE_4_1, exp: &EXP_4_1 },
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
    // @todo: test different mappers!
}