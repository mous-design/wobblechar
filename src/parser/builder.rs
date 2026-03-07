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
impl<const N: usize, M> Builder<'_, N, M> {
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
    pub fn try_build(self) -> Result<Parser<'a, N, M>, ParseError> {
        Parser::new(self.string, self.mapper)
    }
    pub fn build(self) -> Parser<'a, N, M> {
        self.try_build().unwrap()
    }

}
