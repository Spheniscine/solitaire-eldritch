use std::iter::Peekable;

pub struct Runs<I: Iterator> {
    peeker: Peekable<I>
}
impl<T: PartialEq, I: Iterator<Item = T>> Runs<I> {
    fn from_iter(iter: I) -> Self { Self { peeker: iter.peekable() } }
}
impl<T: PartialEq, I: Iterator<Item = T>> Iterator for Runs<I> {
    type Item = (T, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.peeker.next()?;
        let mut count: usize = 1;
        loop {
            let next = self.peeker.peek();
            if next.map(|x| *x != item).unwrap_or(true) { return Some((item, count)); }
            count += 1;
            self.peeker.next();
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) { (0, self.peeker.size_hint().1) }
}
pub trait RunsTrait<T: PartialEq, I: Iterator<Item = T>> {
    fn runs(self) -> Runs<I>;
}
impl<T: PartialEq, I: Iterator<Item = T>> RunsTrait<T, I> for I {
    fn runs(self) -> Runs<I> { Runs::from_iter(self) }
}