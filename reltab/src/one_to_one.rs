use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast_mut, cast_slice_mut};

use slice_rbtree::{init_tree, Error, RBTree};

pub const MAGIC: &[u8; 18] = b"OneToOne container";

#[derive(Debug)]
// TODO: implement a proper Debug
pub struct OneToOne<'a, K, V, const KSIZE: usize, const VSIZE: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: Ord + BorshDeserialize + BorshSerialize,
{
    direct_relation: RBTree<'a, K, V, KSIZE, VSIZE>,
    converse_relation: RBTree<'a, V, K, VSIZE, KSIZE>,
    magic: &'a [u8],
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> OneToOne<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: Ord + BorshDeserialize + BorshSerialize,
{
    pub fn init_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() < MAGIC.len() {
            return Err(Error::TooSmall);
        }

        let (magic, tail) = slice.split_at_mut(MAGIC.len());

        if tail.len() % 2 != 0 {
            return Err(Error::WrongNodePoolSize);
        }

        let (direct, converse) = tail.split_at_mut(tail.len() / 2);

        let direct_relation = RBTree::<'a, K, V, KSIZE, VSIZE>::init_slice(direct)?;
        let converse_relation = RBTree::<'a, V, K, VSIZE, KSIZE>::init_slice(converse)?;

        magic.copy_from_slice(MAGIC);

        Ok(Self {
            direct_relation,
            converse_relation,
            magic,
        })
    }

    pub unsafe fn from_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() < MAGIC.len() {
            return Err(Error::TooSmall);
        }

        let (magic, tail) = slice.split_at_mut(MAGIC.len());

        if magic != MAGIC {
            return Err(Error::WrongMagic);
        }

        if tail.len() % 2 != 0 {
            return Err(Error::WrongNodePoolSize);
        }

        let (direct, converse) = tail.split_at_mut(tail.len() / 2);

        unsafe {
            let direct_relation = RBTree::<'a, K, V, KSIZE, VSIZE>::from_slice(direct)?;
            let converse_relation = RBTree::<'a, V, K, VSIZE, KSIZE>::from_slice(converse)?;

            Ok(Self {
                direct_relation,
                converse_relation,
                magic,
            })
        }
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
