use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::cmp::Ord;

use super::{Error, KeysIterator, PairsIterator, RBForest, ValuesIterator};

//#[derive(Debug)]
pub struct RBTree<'a, K, V, const KSIZE: usize, const VSIZE: usize>(
    RBForest<'a, K, V, KSIZE, VSIZE, 1>,
)
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize;

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> RBTree<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
{
    pub fn init_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        RBForest::<'a, K, V, KSIZE, VSIZE, 1>::init_slice(slice).map(|tree| Self(tree))
    }

    #[must_use]
    pub fn expected_size(num_entries: usize) -> usize {
        RBForest::<'a, K, V, KSIZE, VSIZE, 1>::expected_size(num_entries)
    }

    pub unsafe fn from_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        unsafe { RBForest::<'a, K, V, KSIZE, VSIZE, 1>::from_slice(slice).map(|tree| Self(tree)) }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len(0)
    }

    pub fn clear(&mut self) {
        self.0.clear()
    }

    #[must_use]
    pub fn free_nodes_left(&self) -> usize {
        self.0.free_nodes_left()
    }

    #[must_use]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.contains_key(0, k)
    }

    #[must_use]
    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.get_key_value(0, k)
    }

    #[must_use]
    pub fn get<Q>(&self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.get(0, k)
    }

    pub fn insert(&mut self, k: K, v: V) -> Result<Option<V>, Error> {
        self.0.insert(0, k, v)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty(0)
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.remove(0, key)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.remove_entry(0, key)
    }

    /// Deletes entry without deserializing the value.
    ///
    /// Return `true` if there was a value with the given `key`.
    pub fn delete<Q>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.0.delete(0, key)
    }

    pub fn first_entry(&self) -> Option<(K, V)> {
        self.0.first_entry(0)
    }

    pub fn last_entry(&self) -> Option<(K, V)> {
        self.0.last_entry(0)
    }

    pub fn pairs<'b>(&'b self) -> PairsIterator<'b, 'a, K, V, KSIZE, VSIZE, 1> {
        self.0.pairs(0)
    }

    pub fn keys<'b>(&'b self) -> KeysIterator<'b, 'a, K, V, KSIZE, VSIZE, 1> {
        self.0.keys(0)
    }

    pub fn values<'b>(&'b self) -> ValuesIterator<'b, 'a, K, V, KSIZE, VSIZE, 1> {
        self.0.values(0)
    }
}

#[cfg(test)]
mod tests;
