use borsh::{BorshDeserialize, BorshSerialize};

use slice_rbtree::{Error, RBTree};

pub struct OneToOne<'a, K, V, const KSIZE: usize, const VSIZE: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: Ord + BorshDeserialize + BorshSerialize,
{
    direct_relation: RBTree<'a, K, V, KSIZE, VSIZE>,
    converse_relation: RBTree<'a, V, K, VSIZE, KSIZE>,
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> OneToOne<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: Ord + BorshDeserialize + BorshSerialize,
{
    pub fn init_slice(slice: &mut [u8]) -> Self {
        unimplemented!();
    }

    pub unsafe fn from_slice(slice: &mut [u8]) -> Self {
        unimplemented!();
    }

    pub fn get_value(key: &K) -> Option<V> {
        unimplemented!();
    }

    pub fn get_key(val: &V) -> Option<K> {
        unimplemented!();
    }

    pub fn insert(key: &K, val: &V) -> Result<(K, V), Error> {
        unimplemented!();
    }

    pub fn remove_by_key(key: &K, val: &V) -> Option<V> {
        unimplemented!();
    }

    pub fn remove_by_value(val: &V) -> Option<K> {
        unimplemented!();
    }
}
