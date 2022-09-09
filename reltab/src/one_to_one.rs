use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::fmt;

use slice_rbtree::{tree_size, Error, KeysIterator, RBTree};

pub const MAGIC: &[u8; 18] = b"OneToOne container";

#[must_use]
#[inline]
pub fn one_to_one_size(k_size: usize, v_size: usize, max_nodes: usize) -> usize {
    2 * tree_size(k_size, v_size, max_nodes) + MAGIC.len()
}

pub struct OneToOne<'a, K, V, const KSIZE: usize, const VSIZE: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize + Clone,
    V: Ord + BorshDeserialize + BorshSerialize + Clone,
{
    direct_relation: RBTree<'a, K, V, KSIZE, VSIZE>,
    converse_relation: RBTree<'a, V, K, VSIZE, KSIZE>,
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> OneToOne<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize + Clone,
    V: Ord + BorshDeserialize + BorshSerialize + Clone,
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
            })
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.direct_relation.len()
    }

    pub fn clear(&mut self) {
        self.direct_relation.clear();
        self.converse_relation.clear();
    }

    #[must_use]
    pub fn free_nodes_left(&self) -> usize {
        self.direct_relation.free_nodes_left()
    }

    #[must_use]
    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.contains_key(k)
    }

    #[must_use]
    pub fn contains_value<Q>(&self, v: &Q) -> bool
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.converse_relation.contains_key(v)
    }

    #[must_use]
    pub fn get_key_value_by_key<Q>(&self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.get_key_value(k)
    }

    #[must_use]
    pub fn get_key_value_by_value<Q>(&self, v: &Q) -> Option<(K, V)>
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.converse_relation.get_key_value(v).map(|(v, k)| (k, v))
    }

    #[must_use]
    pub fn get_value<Q>(&self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.get(k)
    }

    #[must_use]
    pub fn get_key<Q>(&self, v: &Q) -> Option<K>
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.converse_relation.get(v)
    }

    pub fn insert(&mut self, k: K, v: V) -> Result<Option<(K, V)>, Error> {
        let old_val = self.direct_relation.insert(k.clone(), v.clone())?;
        match self.converse_relation.insert(v, k.clone()) {
            Ok(Some(old_key)) => Ok(Some((
                old_key,
                old_val.expect("data corruption: direct and converse containers are out of sync"),
            ))),
            Ok(None) => {
                debug_assert!(old_val.is_none());
                Ok(None)
            }
            Err(e) => {
                // second insertion failed, so we have to revert the first one
                if let Some(val) = old_val {
                    self.direct_relation
                        .insert(k, val)
                        .expect("change revertion failed, the container is now corrupted");
                }
                Err(e)
            }
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.direct_relation.is_empty()
    }

    pub fn remove_by_key<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let val = self.direct_relation.remove(key);
        if let Some(ref val) = val {
            let result = self.converse_relation.remove(val);
            debug_assert!(result.is_some());
        }
        val
    }

    pub fn remove_by_value<Q>(&mut self, value: &Q) -> Option<K>
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let key = self.converse_relation.remove(value);
        if let Some(ref key) = key {
            let result = self.direct_relation.remove(key);
            debug_assert!(result.is_some());
        }
        key
    }

    pub fn remove_entry_by_key<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.remove(key).map(|val| {
            let key = self.converse_relation.remove(&val);
            (
                key.expect("data corruption: direct and converse containers are out of sync"),
                val,
            )
        })
    }

    pub fn remove_entry_by_value<Q>(&mut self, value: &Q) -> Option<(K, V)>
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.converse_relation.remove(value).map(|key| {
            let val = self.direct_relation.remove(&key);
            (
                key,
                val.expect("data corruption: direct and converse containers are out of sync"),
            )
        })
    }

    #[must_use]
    pub fn keys<'b>(&'b self) -> KeysIterator<'b, 'a, K, V, KSIZE, VSIZE> {
        self.direct_relation.keys()
    }

    #[must_use]
    pub fn values<'b>(&'b self) -> KeysIterator<'b, 'a, V, K, VSIZE, KSIZE> {
        self.converse_relation.keys()
    }
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> fmt::Debug
    for OneToOne<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize + fmt::Debug + Clone,
    V: Ord + BorshDeserialize + BorshSerialize + fmt::Debug + Clone,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(self.direct_relation.pairs()).finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn init() {
        let mut slice = vec![0; one_to_one_size(4, 4, 100)];

        OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        unsafe {
            OneToOne::<u32, u32, 4, 4>::from_slice(&mut slice).unwrap();
        }
    }

    #[test]
    fn add_value() {
        let mut slice = vec![0; one_to_one_size(4, 4, 100)];

        let mut container = OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        container.insert(1, 6).unwrap();
        container.insert(2, 7).unwrap();
        container.insert(3, 8).unwrap();
        container.insert(4, 9).unwrap();
        container.insert(5, 10).unwrap();

        assert_eq!(container.get_value(&1), Some(6));
        assert_eq!(container.get_value(&2), Some(7));
        assert_eq!(container.get_value(&3), Some(8));
        assert_eq!(container.get_value(&4), Some(9));
        assert_eq!(container.get_value(&5), Some(10));
        assert_eq!(container.get_value(&6), None);

        assert_eq!(container.get_key(&6), Some(1));
        assert_eq!(container.get_key(&7), Some(2));
        assert_eq!(container.get_key(&8), Some(3));
        assert_eq!(container.get_key(&9), Some(4));
        assert_eq!(container.get_key(&10), Some(5));
        assert_eq!(container.get_key(&1), None);
    }
}
