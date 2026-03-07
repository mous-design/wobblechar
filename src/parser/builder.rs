use super::{Parser, Mapper, BoolMapper, NumMapper};
use super::mapper::{Entry, default, hash, constant};
use super::line_parser::ParseError;
use num_traits::Num;
pub struct NoMapper;

/// Builder for constructing a [`Parser`].
///
/// `N` is the number of output values (channels) the parsed needs to produce.
/// Start with [`new_from_string`](Builder::new_from_string), chain a mapper
/// method to select the character-to-value mapping, then call
/// [`build`](Builder::build) to obtain an iterator.
///
/// # Example
/// ```
/// use wobblechar::Builder;
/// let parser = Builder::<1>::new_from_string("_|‾|_")
///     .with_def_bool_mapper()
///     .build();
/// ```
pub struct Builder<'a, const N:usize, M = NoMapper>
{
    string: &'a str,
    mapper: M,
}
impl<'a, const N: usize> Builder<'a, N> {
    /// Creates a new `Builder` from a waveform string.
    ///
    /// Multiple lines are separated by `\n`. Leading and trailing whitespace
    /// per line is ignored, as are lines that are empty or consist only of
    /// a `#` comment. A line may carry a label prefix (`Name: content`) to
    /// allow non-contiguous continuation across blocks.
    pub fn new_from_string(string: &'a str) -> Self {
        Self { string, mapper: NoMapper }
    }

    /// Uses the default boolean mapper.
    ///
    /// Character mapping: `'_'` → `false`, `'|'` → toggle, any other character → `true`.
    pub fn with_def_bool_mapper(self) -> Builder<'a, N, BoolMapper<default::LookupBool>> {
        let lookup = default::LookupBool::new();
        let mapper =  BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }

    /// Uses the default numeric mapper with output type `T`.
    ///
    /// Character mapping: `'_'` → `T::zero()`, `'|'` → toggle, any other character → `T::one()`.
    pub fn with_def_num_mapper<T>(self) -> Builder<'a, N, NumMapper<default::LookupNum<T>>>
        where T: Num + Copy + Ord,
    {
        let lookup = default::LookupNum::<T>::new();
        let mapper = NumMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }

    /// Uses a boolean mapper backed by a `HashMap`.
    ///
    /// Characters not present in `map` are treated as unknown and will stop
    /// the parser when encountered.
    #[cfg(feature = "std")]
    pub fn with_hash_bool_mapper<'b>(self, map:&'b std::collections::HashMap<char, Entry<bool>>)
        -> Builder<'a, N, BoolMapper<hash::LookupBool<'b>>>
    {
        let lookup = hash::LookupBool::new(map);
        let mapper = BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }

    /// Uses a numeric mapper with output type `T` backed by a `HashMap`.
    ///
    /// `min` and `max` are derived from the smallest and largest values in `map`.
    /// Characters not present in `map` are treated as unknown and will stop
    /// the parser when encountered.
    #[cfg(feature = "std")]
    pub fn with_hash_num_mapper<'b, T>(self, map:&'b std::collections::HashMap<char, Entry<T>>)
        -> Builder<'a, N, NumMapper<hash::LookupNum<'b, T>>>
        where T: Num + Copy + Ord,
    {
        let lookup = hash::LookupNum::<T>::new(map);
        let mapper = NumMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }

    /// Uses a boolean mapper backed by a compile-time slice of `(char, Entry<bool>)` pairs.
    ///
    /// Suitable for `no_std` environments. Characters not present in `map` are
    /// treated as unknown and will stop the parser when encountered.
    pub fn with_const_bool_map<'b>(self, map:&'b [(char, Entry<bool>)])
        -> Builder<'a, N, BoolMapper<constant::LookupBool<'b>>>
    {
        let lookup = constant::LookupBool::new(map);
        let mapper = BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }

    /// Uses a numeric mapper with output type `T` backed by a compile-time slice
    /// of `(char, Entry<T>)` pairs.
    /// This mapper allows you to have any number of values, for example for 
    /// tri-state logic.
    ///
    /// `min` and `max` are derived from the smallest and largest values in `map`.
    /// Suitable for `no_std` environments. Characters not present in `map` are
    /// treated as unknown and will stop the parser when encountered.
    pub fn with_const_num_mapper<'b, T>(self, map:&'b [(char, Entry<T>)])
        -> Builder<'a, N, NumMapper<constant::LookupNum<'b,T>>>
        where T: Num + Copy + Ord,
    {
        let lookup = constant::LookupNum::<T>::new(map);
        let mapper = NumMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }
}
impl<'a, const N: usize, M> Builder<'a, N, M>
where M:Mapper
{
    /// Builds the parser, returning an error if the input cannot be parsed
    /// (e.g. the number of output lines exceeds `N`).
    pub fn try_build(self) -> Result<Parser<'a, N, M>, ParseError> {
        Parser::new(self.string, self.mapper)
    }

    /// Builds the parser. Panics if the input cannot be parsed. Good for testing.
    ///
    /// Use [`try_build`](Builder::try_build) to handle errors explicitly.
    pub fn build(self) -> Parser<'a, N, M> {
        self.try_build().unwrap()
    }

}
