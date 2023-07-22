pub struct Padded<I>
where
    I: Iterator,
    I::Item: Clone,
{
    iter: std::iter::Peekable<I>,
    separator: I::Item,
    yield_seperator: Option<I::Item>,
}

impl<I> Padded<I>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I, separator: I::Item) -> Self {
        Self {
            iter: iter.peekable(),
            separator,
            yield_seperator: None,
        }
    }
}

impl<I> Iterator for Padded<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        if self.yield_seperator.is_some() {
            self.yield_seperator.take()
        } else {
            self.yield_seperator = Some(self.separator.clone());
            self.iter.next()
        }
    }
}

pub trait PaddedT: Iterator {
    fn padded(self, separator: Self::Item) -> Padded<Self>
    where
        Self: Sized,
        Self::Item: Clone,
    {
        Padded::new(self, separator)
    }
}

impl<I> PaddedT for I where I: Iterator {}
