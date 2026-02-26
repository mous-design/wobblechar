use super::{Parser, LineIters, Mapper, BoolMapper, NumMapper};
use super::mapper::{Entry, default, hash, constant};
use num_traits::Num;
pub struct NoMapper;

pub struct Builder<'a, const N:usize, M = NoMapper>
{
    line_iters: LineIters<'a, N>,
    mapper: M,
}
impl<'a, const N: usize, M> Builder<'a, N, M> {
    pub fn with_mapper(self, mapper: M) -> Self {
        Builder {
            line_iters: self.line_iters,
            mapper,
        }
    }
}
impl<'a, const N: usize> Builder<'a, N> {
    pub fn new_from_string(s: &'a str) -> Self {
        Self { line_iters: Self::string_to_iters(s), mapper: NoMapper }
    }
    // have a vec of iterators per line, with the already trimmed lines.
    // @todo: nieuw idee: alle regels volledig trim doen, split op newline (of 
    //        variabele delim, maar dat lijkt me niet echt nodig). Lege regels negeren.
    //        Dan labels ondersteunen. Als label vaker terugkomt, dan chainen
    //        we de iterator.
    //        Dan kan dit dus:
    // 
    // D0:  _|‾‾|___
    // D1:  ___|‾‾|_
    //
    // D0:  |‾‾|___
    // D1:  __|‾‾|_
    // 
    // Zonder labels is gewoon elke regel een signaal. Met labels symbol-tabel 
    // bijhoduen. Syntax kan dan zijn: \w+: aan begin regel is label. Lijkt me
    // onnodig om deze syntax instelbaar te maken...
    fn string_to_iters(string: &str) -> LineIters<'_,N>
    {
        string
            .lines().map(|line| line.trim_start().chars().peekable())
            .collect()
    }
    pub fn with_def_bool_mapper(self) -> Builder<'a, N, BoolMapper<default::LookupBool>> {
        let lookup = default::LookupBool::new();
        let mapper =  BoolMapper::new(lookup);
        Builder {
            line_iters: self.line_iters,
            mapper,
        }
    }
    pub fn with_def_num_mapper<T>(self) -> Builder<'a, N, NumMapper<default::LookupNum<T>>>
        where T: Num + Copy + Ord,
    {
        let lookup = default::LookupNum::<T>::new();
        let mapper = NumMapper::new(lookup);
        Builder {
            line_iters: self.line_iters,
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
            line_iters: self.line_iters,
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
            line_iters: self.line_iters,
            mapper,
        }
    }
    pub fn with_const_bool_map(self, map:&'a [(char, Entry<bool>)]) 
        -> Builder<'a, N, BoolMapper<constant::LookupBool<'a>>>
    {
        let lookup = constant::LookupBool::new(map);
        let mapper = BoolMapper::new(lookup);
        Builder {
            line_iters: self.line_iters,
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
            line_iters: self.line_iters,
            mapper,
        }
    }
}
impl<'a, const N: usize, M> Builder<'a, N, M> 
where M:Mapper
{
    pub fn build(self) -> Parser<'a, N, M> {
        Parser::new(self.line_iters, self.mapper)
    }
}

// fn x() {
//     let parser = Builder::<8>::new_from_string("0101\n1111")
//         .with_def_bool_mapper()
//         .build();
//     let parser = Builder::<8>::new_from_string("...")
//         .with_def_num_mapper::<u8>()
//         .build();
// }
