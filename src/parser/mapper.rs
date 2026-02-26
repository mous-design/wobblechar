use num_traits::Num;
#[derive(Clone, Copy)]
pub enum Entry<T> {
    Value(T),
    Toggle,
}
impl <T>Entry<T> {
    fn value(&self) -> Option<&T> {
        match self {
            Entry::Value(v) => Some(v),
            _ => None,
        }
    }
    #[allow(dead_code)]
    fn is_toggle(&self) -> bool {
        matches!(self, Entry::Toggle)
    }
}

pub trait LookupMap
{
    type Out;
    fn get(&self, c: char) -> Option<Entry<Self::Out>>;
    fn min(&self) -> Self::Out;
    fn max(&self) ->Self::Out;
}

pub trait Mapper where <Self::Map as LookupMap>::Out: Copy + PartialEq,
{
    type Map: LookupMap;
    fn map(&self) -> &Self::Map;

    /// The low value for this type.
    fn low(&self) -> <Self::Map as LookupMap>::Out {
        self.map().min()
    }
    /// The high value for this type.
    fn high(&self) -> <Self::Map as LookupMap>::Out {
        self.map().max()
    }

    /// Map a character to value of T. By default map only '_' to low, the rest to high.
    fn value(&self, c: char) -> Option<<Self::Map as LookupMap>::Out> {
        self.map().get(c)?.value().copied()
    }

    /// Is this a special toggle-character?
    fn is_toggle(&self, c: char) -> bool {
        matches!(self.map().get(c), Some(Entry::Toggle))
    }
    
    /// Toggle a value, if applicable.
    fn toggle(&self, v: <Self::Map as LookupMap>::Out) -> <Self::Map as LookupMap>::Out {
        if v == self.low() {
            self.high()
        } else {
            self.low()
        }
    }

    /// Normally, just return low for default.
    fn default(&self) -> <Self::Map as LookupMap>::Out {
        self.low()
    }
}

pub struct BoolMapper<L: LookupMap> {
    map: L
}
impl <L: LookupMap> BoolMapper<L> {
    pub fn new(map: L) -> Self {
        Self { map }
    }
}
impl<L: LookupMap<Out = bool>> Mapper for BoolMapper<L> {
    type Map = L;

    fn map(&self) -> &L { &self.map }
    fn toggle(&self, v: bool) -> bool { !v }
}

pub struct NumMapper<L> where L:LookupMap
{
    map: L
}
impl <L>NumMapper<L> where L:LookupMap
{
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

pub mod default {
    use super::{LookupMap, Entry};
    use core::marker::PhantomData;

    pub struct LookupBool;
    impl LookupBool {
        #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub struct LookupNum<T> where T: Num + Copy + Ord {
        _phantom: PhantomData<T>
    }
    impl<T> LookupNum<T> where T: Num + Copy + Ord {
        #[allow(dead_code)]
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

#[cfg(feature = "std")]
pub mod hash {
    use super::{LookupMap, Entry};
    use std::collections::HashMap;

    // Boolen
    #[allow(dead_code)]
    pub struct LookupBool<'a> {
        map: &'a HashMap<char, Entry<bool>>,
    }
    impl <'a>LookupBool<'a> {
        #[allow(dead_code)]
        pub fn new(map: &'a HashMap<char, Entry<bool>>) -> Self {
            Self { map }
        }
    }
    impl <'a>LookupMap for LookupBool<'_> {
        type Out = bool;

        fn get(&self, c: char) -> Option<Entry<bool>> {
            self.map.get(&c).copied()
        }
        fn min(&self) -> bool { false }
        fn max(&self) -> bool { true }
    }

    // Numeric
    use num_traits::Num;
    #[allow(dead_code)]
    pub struct LookupNum<'a, T> where T: Num + Copy + Ord {
        map: &'a HashMap<char, Entry<T>>,
        min: T,
        max: T,
    }
    impl<'a, T> LookupNum<'a, T> where T: Num + Copy + Ord {
        #[allow(dead_code)]
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

pub mod constant {
    use super::{LookupMap, Entry};

    // const CHAR_MAP: [(char, i8); 3] = [('X', 1), ('-', 0), ('_', -1)];

    // Boolen
    #[allow(dead_code)]
    pub struct LookupBool<'a> {
        map: &'a [(char, Entry<bool>)],
    }
    impl<'a> LookupBool<'a> {
        #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub struct LookupNum<'a, T> where T: Num + Copy + Ord {
        map: &'a [(char, Entry<T>)],
        min: T,
        max: T,
    }
    impl<'a, T> LookupNum<'a, T> where T: Num + Copy + Ord {
        #[allow(dead_code)]
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