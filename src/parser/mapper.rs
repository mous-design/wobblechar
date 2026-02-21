
pub trait Mapper
    where <Self as Mapper>::Output: PartialEq +Copy
{
    type Output;
    /// The low value for this type. This method MUST be implemented.
    fn low(&self) -> Self::Output;
    /// The high value for this type. This method MUST be implemented.
    fn high(&self) -> Self::Output;

    /// Map a character to value of T. By default map only '_' to low, the rest to high.
    fn value(&self, c: char) -> Option<Self::Output> {
        match c {
            '_' => Some(self.low()),
            _ => Some(self.high()),
        }
    }

    /// Is this a special toggle-character?
    fn is_toggle(&self, c: char) -> bool {
        c == '|'
    }
    
    /// Toggle a value, if applicable.
    fn toggle(&self, v: Self::Output) -> Self::Output {
        if v == self.low() {
            self.high()
        } else {
            self.low()
        }
    }

    fn default(&self) -> Self::Output {
        self.low()
    }
}


pub mod default {
    use super::{Mapper};
    
    // Boolean
    pub struct BoolMapper {

    }
    impl BoolMapper {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl Mapper for BoolMapper {
        type Output = bool;
        fn low(&self) -> bool { false }
        fn high(&self) -> bool { true }
    }

    // Numeric
    use num_traits::Num;
    use core::marker::PhantomData;
    pub struct NumMapper<T> where T: Num + Copy
    {
        _phantom: PhantomData<T>,
    }
    impl <T> NumMapper<T> where T: Num + Copy {
        pub fn new() -> Self {
            Self { _phantom: PhantomData }
        }
    }
    impl <T> Mapper for NumMapper<T> 
        where T: Num + Copy
    {
        type Output = T;
        fn low(&self) -> T { T::zero() }
        fn high(&self) -> T { T::one() }
        fn toggle(&self, v: T) -> T {
            self.low() + self.high() - v
        }
    }
}

#[cfg(feature = "std")]
pub mod hash {
    use super::Mapper;
    use std::collections::HashMap;

    // Boolen
    #[allow(dead_code)]
    pub struct BoolMapper<'a> {
        map: &'a HashMap<char, bool>,
    }
    impl<'a> BoolMapper<'a> {
        #[allow(dead_code)]
        pub fn new(map: &'a HashMap<char, bool>) -> Self {
            Self { map }
        }
    }
    impl Mapper for BoolMapper<'_> {
        type Output = bool;
        fn low(&self) -> bool { false }
        fn high(&self) -> bool { true }
        fn value(&self, c: char) -> Option<bool> {
            self.map.get(&c).copied()
        }
        fn toggle(&self, x: bool) -> bool {
            !x 
        }
    }

    // Numeric
    use num_traits::Num;
    #[allow(dead_code)]
    pub struct NumMapper<'a, T> where T: Num + Copy + Ord {
        map: &'a HashMap<char, T>,
        min: T,
        max: T,
    }
    impl<'a, T> NumMapper<'a, T> where T: Num + Copy + Ord {
        #[allow(dead_code)]
        pub fn new(map: &'a HashMap<char, T>) -> Self {
            let min = map.values().copied().reduce(T::min).unwrap_or(T::zero());
            let max = map.values().copied().reduce(T::max).unwrap_or(T::one());
            Self { map, min, max }
        }
    }
    impl <T> Mapper for NumMapper<'_, T> where T: Num + Copy + Ord {
        type Output = T;
        fn low(&self) -> T { self.min }
        fn high(&self) -> T { self.max }
        fn value(&self, c: char) -> Option<T> {
            self.map.get(&c).copied()
        }
        fn toggle(&self, x: T) -> T {
            self.max + self.min - x
        }
    }
}

pub mod constant {
    use super::Mapper;

    // const CHAR_MAP: [(char, i8); 3] = [('X', 1), ('-', 0), ('_', -1)];

    // Boolen
    #[allow(dead_code)]
    pub struct BoolMapper<'a> {
        map: &'a [(char, bool)],
    }
    impl<'a> BoolMapper<'a> {
        #[allow(dead_code)]
        pub fn new(map: &'a [(char, bool)]) -> Self {
            Self { map }
        }
    }
    impl Mapper for BoolMapper<'_> {
        type Output = bool;
        fn low(&self) -> bool { false }
        fn high(&self) -> bool { true }
        fn value(&self, c: char) -> Option<bool> {
            self.map.iter().find(|(ch, _)| *ch == c).map(|(_, b)| *b)
        }
        fn toggle(&self, x: bool) -> bool { !x }
    }

    // Integers
    use num_traits::Num;
    #[allow(dead_code)]
    pub struct NumMapper<'a, T> where T: Num + Copy + Ord {
        map: &'a [(char, T)],
        min: T,
        max: T,
    }
    impl<'a, T> NumMapper<'a, T> where T: Num + Copy + Ord {
        #[allow(dead_code)]
        pub fn new(map: &'a [(char, T)]) -> Self {
            let min = map.iter().map(|(_, b)| *b).min().unwrap_or(T::zero());
            let max = map.iter().map(|(_, b)| *b).max().unwrap_or(T::one());
            Self { map, min, max }
        }
    }
    impl <T> Mapper for NumMapper<'_, T> where T: Num + Copy + Ord {
        type Output = T;
        fn low(&self) -> T { self.min }
        fn high(&self) -> T { self.max }
        fn value(&self, c: char) -> Option<T> {
            self.map.iter().find(|(ch, _)| *ch == c).map(|(_, b)| *b)
        }

        fn toggle(&self, x: T) -> T {
            self.max + self.min - x
        }
    }
}
