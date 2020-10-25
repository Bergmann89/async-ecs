use std::iter::Iterator;

use asparit::{Consumer, Executor, ParallelIterator, Producer, Reducer, WithSetup};

use crate::misc::{BitIter, BitProducer};

use super::Join;

/* ParJoin */

pub trait ParJoin: Join {
    fn par_join(self) -> JoinParIter<Self>
    where
        Self: Sized,
    {
        if <Self as Join>::is_unconstrained() {
            log::warn!(
                "`ParJoin` possibly iterating through all indices, you might've made a join with all `MaybeJoin`s, which is unbounded in length."
            );
        }

        JoinParIter(self)
    }
}

/* JoinParIter */

pub struct JoinParIter<J>(J);

impl<'a, J> ParallelIterator<'a> for JoinParIter<J>
where
    J: Join + Send + 'a,
    J::Type: Send,
    J::Value: Copy + Send,
    J::Mask: Copy + Send + Sync,
{
    type Item = J::Type;

    fn drive<E, C, D, R>(self, executor: E, consumer: C) -> E::Result
    where
        E: Executor<'a, D>,
        C: Consumer<Self::Item, Result = D, Reducer = R> + 'a,
        D: Send + 'a,
        R: Reducer<D> + Send + 'a,
    {
        let (keys, values) = self.0.open();

        let keys = BitIter::new(keys);

        let producer = BitProducer::new(keys);
        let producer = JoinProducer::<J>::new(producer, values);

        executor.exec(producer, consumer)
    }
}

/* JoinProducer */

struct JoinProducer<J>
where
    J: Join,
{
    keys: BitProducer<J::Mask>,
    values: J::Value,
}

impl<J> JoinProducer<J>
where
    J: Join,
{
    fn new(keys: BitProducer<J::Mask>, values: J::Value) -> Self {
        JoinProducer { keys, values }
    }
}

unsafe impl<J> Send for JoinProducer<J>
where
    J: Join + Send,
    J::Type: Send,
    J::Value: Send,
    J::Mask: Send + Sync,
{
}

impl<J> WithSetup for JoinProducer<J> where J: Join {}

impl<J> Producer for JoinProducer<J>
where
    J: Join + Send,
    J::Type: Send,
    J::Value: Copy + Send,
    J::Mask: Copy + Send + Sync,
{
    type Item = J::Type;
    type IntoIter = JoinIter<J>;

    fn into_iter(self) -> Self::IntoIter {
        JoinIter {
            keys: self.keys.into_iter(),
            values: self.values,
        }
    }

    fn split(self) -> (Self, Option<Self>) {
        let values = self.values;
        let (left, right) = self.keys.split();

        let left = JoinProducer::new(left, values);
        let right = right.map(|right| JoinProducer::new(right, values));

        (left, right)
    }
}

/* JoinIter */

struct JoinIter<J>
where
    J: Join,
{
    keys: BitIter<J::Mask>,
    values: J::Value,
}

impl<J> Iterator for JoinIter<J>
where
    J: Join,
{
    type Item = J::Type;

    fn next(&mut self) -> Option<Self::Item> {
        let index = self.keys.next()?;
        let value = J::get(&mut self.values, index);

        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.keys.size_hint()
    }
}
