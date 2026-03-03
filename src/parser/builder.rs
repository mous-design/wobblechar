use super::{Parser, Mapper, BoolMapper, NumMapper};
use super::mapper::{Entry, default, hash, constant};
use super::line_parser::ParseError;
use num_traits::Num;
pub struct NoMapper;

pub struct Builder<'a, const N:usize, M = NoMapper>
{
    string: &'a str,
    mapper: M,
}
impl<'a, const N: usize, M> Builder<'a, N, M> {
    pub fn with_mapper(self, mapper: M) -> Self {
        Builder {
            string: self.string,
            mapper,
        }
    }
}

impl<'a, const N: usize> Builder<'a, N> {
    pub fn new_from_string(string: &'a str) -> Self {
        Self { string, mapper: NoMapper }
    }

    pub fn with_def_bool_mapper(self) -> Builder<'a, N, BoolMapper<default::LookupBool>> {
        let lookup = default::LookupBool::new();
        let mapper =  BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }
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
    #[cfg(feature = "std")]
    pub fn with_hash_bool_mapper(self, map:&'a std::collections::HashMap<char, Entry<bool>>)
        -> Builder<'a, N, BoolMapper<hash::LookupBool<'a>>>
    {
        let lookup = hash::LookupBool::new(map);
        let mapper = BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }
    #[cfg(feature = "std")]
    pub fn with_hash_num_mapper<T>(self, map:&'a std::collections::HashMap<char, Entry<T>>) 
        -> Builder<'a, N, NumMapper<hash::LookupNum<'a, T>>>
        where T: Num + Copy + Ord,
    {
        let lookup = hash::LookupNum::<T>::new(map);
        let mapper = NumMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }
    pub fn with_const_bool_map(self, map:&'a [(char, Entry<bool>)]) 
        -> Builder<'a, N, BoolMapper<constant::LookupBool<'a>>>
    {
        let lookup = constant::LookupBool::new(map);
        let mapper = BoolMapper::new(lookup);
        Builder {
            string: self.string,
            mapper,
        }
    }
    pub fn with_const_num_mapper<T>(self, map:&'a [(char, Entry<T>)]) 
        -> Builder<'a, N, NumMapper<constant::LookupNum<'a,T>>> 
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
    pub fn try_build(self) -> Result<Parser<'a, N, M>, ParseError> {
        Parser::new(self.string, self.mapper)
    }
    pub fn build(self) -> Parser<'a, N, M> {
        self.try_build().unwrap()
    }

}
