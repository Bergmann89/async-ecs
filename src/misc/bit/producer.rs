use asparit::{Producer, WithSetup};
use hibitset::BitSetLike;

use super::BitIter;

pub struct BitProducer<T> {
    pub iter: BitIter<T>,
}

impl<T> BitProducer<T> {
    pub fn new(iter: BitIter<T>) -> Self {
        Self { iter }
    }
}

impl<T> WithSetup for BitProducer<T> {}

impl<T> Producer for BitProducer<T>
where
    T: BitSetLike + Copy + Send + Sync,
{
    type Item = u32;
    type IntoIter = BitIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter
    }

    fn split(self) -> (Self, Option<Self>) {
        let (left, right) = self.iter.split();

        (Self::new(left), right.map(Self::new))
    }
}
