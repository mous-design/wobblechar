use num_traits::Num;

/// A character's decoded meaning: either a concrete value or a toggle marker.
///
/// Used in custom character maps passed to
/// [`with_const_bool_map`](crate::Builder::with_const_bool_map),
/// [`with_const_num_mapper`](crate::Builder::with_const_num_mapper), and
/// the `HashMap`-based mapper methods.
#[derive(Clone, Copy)]
pub enum Entry<T> {
    /// The character maps directly to this value.
    Value(T),
    /// The character is an edge marker: toggles the previous value.
    Toggle,
}
impl <T>Entry<T> {
    fn value(&self) -> Option<&T> {
        match self {
            Entry::Value(v) => Some(v),
            _ => None,
        }
    }

}

/// Maps characters to [`Entry`] values and defines the low/high bounds.
///
/// Implement this to create a custom character mapping for use with
/// [`BoolMapper`] or [`NumMapper`].
pub trait LookupMap
{
    /// The output value type (e.g. `bool`, `u8`, `i16`).
    type Out;
    /// Returns the [`Entry`] for `c`, or `None` if `c` is unknown.
    fn get(&self, c: char) -> Option<Entry<Self::Out>>;
    /// The minimum (low) value; used as the default and the `false`/`0` state.
    fn min(&self) -> Self::Out;
    /// The maximum (high) value; used as the `true`/`1` state.
    fn max(&self) -> Self::Out;
}

/// Drives the parser's character decoding and toggle logic.
///
/// The two built-in implementations are [`BoolMapper`] and [`NumMapper`].
/// Implement this trait (together with [`LookupMap`]) to create fully custom
/// decoding behaviour.
pub trait Mapper where <Self::Map as LookupMap>::Out: Copy + PartialEq,
{
    /// The [`LookupMap`] used to decode individual characters.
    type Map: LookupMap;

    /// Returns a reference to the underlying [`LookupMap`].
    fn map(&self) -> &Self::Map;

    /// The low (minimum) value; delegates to [`LookupMap::min`].
    fn low(&self) -> <Self::Map as LookupMap>::Out {
        self.map().min()
    }

    /// The high (maximum) value; delegates to [`LookupMap::max`].
    fn high(&self) -> <Self::Map as LookupMap>::Out {
        self.map().max()
    }

    /// Returns the concrete value for `c`, or `None` if `c` maps to [`Entry::Toggle`] or is unknown.
    fn value(&self, c: char) -> Option<<Self::Map as LookupMap>::Out> {
        self.map().get(c)?.value().copied()
    }

    /// Returns `true` if `c` is a toggle character according to the [`LookupMap`].
    fn is_toggle(&self, c: char) -> bool {
        matches!(self.map().get(c), Some(Entry::Toggle))
    }

    /// Returns the toggled counterpart of `v`.
    ///
    /// The default implementation switches between `low` and `high`.
    fn toggle(&self, v: <Self::Map as LookupMap>::Out) -> <Self::Map as LookupMap>::Out {
        if v == self.low() {
            self.high()
        } else {
            self.low()
        }
    }

    /// The value used when a line has no previous state; defaults to `low`.
    fn default(&self) -> <Self::Map as LookupMap>::Out {
        self.low()
    }
}

/// A [`Mapper`] for `bool` output values.
///
/// Toggle is implemented as logical negation (`!v`), since arithmetic on `bool` is not allowed in Rust.
pub struct BoolMapper<L: LookupMap> {
    map: L
}
impl <L: LookupMap> BoolMapper<L> {
    /// Creates a `BoolMapper` wrapping the given [`LookupMap`].
    pub fn new(map: L) -> Self {
        Self { map }
    }
}
impl<L: LookupMap<Out = bool>> Mapper for BoolMapper<L> {
    type Map = L;

    fn map(&self) -> &L { &self.map }
    fn toggle(&self, v: bool) -> bool { !v }
}

/// A [`Mapper`] for numeric output values.
///
/// Toggle is implemented as `min + max - v`, which correctly mirrors any
/// value between the two extremes defined by the [`LookupMap`].
pub struct NumMapper<L> where L:LookupMap
{
    map: L
}
impl <L>NumMapper<L> where L:LookupMap
{
    /// Creates a `NumMapper` wrapping the given [`LookupMap`].
    pub fn new(map: L) -> Self {
        Self { map }
    }
}
impl<L> Mapper for NumMapper<L>
    where
        L: LookupMap,
        L::Out: Num + Copy + Ord,
{
    type Map = L;

    fn map(&self) -> &L { &self.map }
    fn toggle(&self, v: L::Out) -> L::Out {
        self.low() + self.high() - v
    }
}

/// Default [`LookupMap`] implementations using the built-in character set.
///
/// `'_'` → low, `'|'` → toggle, anything else → high.
pub mod default {
    use super::{LookupMap, Entry};
    use core::marker::PhantomData;

    pub struct LookupBool;
    impl LookupBool {
        pub fn new() -> Self {
            Self {}
        }
    }
    impl LookupMap for LookupBool {
        type Out = bool;
        fn get(&self, c: char) -> Option<Entry<bool>> {
            Some(match c {
                '_' => Entry::Value(self.min()),
                '|' => Entry::Toggle,
                _ => Entry::Value(self.max()),
            })
        }
        fn min(&self) -> bool { false }
        fn max(&self) -> bool { true }
    }

    use num_traits::Num;
    pub struct LookupNum<T> where T: Num + Copy + Ord {
        _phantom: PhantomData<T>
    }
    impl<T> LookupNum<T> where T: Num + Copy + Ord {
        pub fn new() -> Self {
            Self { _phantom: PhantomData }
        }
    }
    impl <T> LookupMap for LookupNum<T> where T: Num + Copy + Ord {
        type Out = T;
        fn get(&self, c: char) -> Option<Entry<T>> {
            Some(match c {
                '_' => Entry::Value(self.min()),
                '|' => Entry::Toggle,
                _ => Entry::Value(self.max()),
            })
        }
        fn min(&self) -> T { T::zero() }
        fn max(&self) -> T { T::one() }
    }
}

/// [`LookupMap`] implementations backed by a [`std::collections::HashMap`].
///
/// Requires the `std` feature. Allows fully dynamic character mappings at runtime.
#[cfg(feature = "std")]
pub mod hash {
    use super::{LookupMap, Entry};
    use std::collections::HashMap;

    // Boolen
    pub struct LookupBool<'a> {
        map: &'a HashMap<char, Entry<bool>>,
    }
    impl <'a>LookupBool<'a> {
        pub fn new(map: &'a HashMap<char, Entry<bool>>) -> Self {
            Self { map }
        }
    }
    impl LookupMap for LookupBool<'_> {
        type Out = bool;

        fn get(&self, c: char) -> Option<Entry<bool>> {
            self.map.get(&c).copied()
        }
        fn min(&self) -> bool { false }
        fn max(&self) -> bool { true }
    }

    // Numeric
    use num_traits::Num;
    pub struct LookupNum<'a, T> where T: Num + Copy + Ord {
        map: &'a HashMap<char, Entry<T>>,
        min: T,
        max: T,
    }
    impl<'a, T> LookupNum<'a, T> where T: Num + Copy + Ord {
        pub fn new(map: &'a HashMap<char, Entry<T>>) -> Self {
            let min = map.values().filter_map(|v| v.value().copied())
                .reduce(T::min).unwrap_or(T::zero());
            let max = map.values().filter_map(|v| v.value().copied())
                .reduce(T::max).unwrap_or(T::one());
            Self { map, min, max }
        }
    }
    impl <T> LookupMap for LookupNum<'_, T> where T: Num + Copy + Ord {
        type Out = T;
        fn get(&self, c: char) -> Option<Entry<T>> {
            self.map.get(&c).copied()
        }
        fn min(&self) -> T { self.min }
        fn max(&self) -> T { self.max }
    }
}

/// [`LookupMap`] implementations backed by a static slice of `(char, Entry<T>)` pairs.
///
/// Suitable for `no_std` environments. The slice can be a `const` or a `static`.
/// See example folder for usage.
pub mod constant {
    use super::{LookupMap, Entry};

    // const CHAR_MAP: [(char, i8); 3] = [('X', 1), ('-', 0), ('_', -1)];

    // Boolen
    pub struct LookupBool<'a> {
        map: &'a [(char, Entry<bool>)],
    }
    impl<'a> LookupBool<'a> {
        pub fn new(map: &'a [(char, Entry<bool>)]) -> Self {
            Self { map }
        }
    }
    
    impl LookupMap for LookupBool<'_> {
        type Out = bool;
    
        fn get(&self, c: char) -> Option<Entry<bool>> {
            self.map.iter().find(|(ch, _)| *ch == c).map(|(_, b)| *b)
        }
        fn min(&self) -> bool { false }
        fn max(&self) -> bool { true }
    }

    // Integers
    use num_traits::Num;
    pub struct LookupNum<'a, T> where T: Num + Copy + Ord {
        map: &'a [(char, Entry<T>)],
        min: T,
        max: T,
    }
    impl<'a, T> LookupNum<'a, T> where T: Num + Copy + Ord {
        pub fn new(map: &'a [(char, Entry<T>)]) -> Self {
            let min = map.iter().filter_map(|(_,v)| v.value().copied())
                .reduce(T::min).unwrap_or(T::zero());
            let max = map.iter().filter_map(|(_,v)| v.value().copied())
                .reduce(T::max).unwrap_or(T::one());
            Self { map, min, max }
        }
    }
    impl <T> LookupMap for LookupNum<'_, T> where T: Num + Copy + Ord {
        type Out = T;
        fn min(&self) -> T { self.min }
        fn max(&self) -> T { self.max }
        fn get(&self, c: char) -> Option<Entry<T>> {
            self.map.iter().find(|(ch, _)| *ch == c).map(|(_, b)| *b)
        }
    }
}