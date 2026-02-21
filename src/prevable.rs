pub struct Prevable<I: Iterator> {
    inner: I,
    prev: Option<I::Item>,
}

impl<I: Iterator> Prevable<I> {
    pub fn new(iter: I) -> Self {
        Self { inner: iter, prev: None }
    }

    pub fn prev(&self) -> Option<I::Item>
        where I::Item: Copy,
    {
        self.prev
    }
}

impl<I: Iterator> Iterator for Prevable<I>
    where I::Item: Copy, 
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.inner.next();
        // Only update prev is real value. So after last next() it is still available.
        if current.is_some() {
            self.prev = current;
        }
        current
    }
}

pub trait PrevableExt: Iterator {
    fn prevable(self) -> Prevable<Self>
    where
        Self: Sized,
    {
        Prevable::new(self)
    }
}

// Implement extension trait for all iterators.
impl<T: Iterator> PrevableExt for T {}
