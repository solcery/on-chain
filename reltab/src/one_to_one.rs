//! This container represents a one to one ([bijective](https://en.wikipedia.org/wiki/Bijection))
//! relation.
//!
//! # Internal structure
//! [`OneToOne`] container consists of three parts:
//! - 18 bytes magic string, must be equal to [`MAGIC`]
//! - `K -> V` [`RBTree`]
//! - `V -> K` [`RBTree`]
//!
//! Both trees have the same size
use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::fmt;

use slice_rbtree::tree::{tree_size, KeysIterator, RBTree, TreeParams};
use slice_rbtree::Error;

pub const MAGIC: &[u8; 18] = b"OneToOne container";

/// Returns the required size of the slice
#[must_use]
#[inline]
pub fn one_to_one_size(k_size: usize, v_size: usize, max_nodes: usize) -> usize {
    2 * tree_size(TreeParams { k_size, v_size }, max_nodes) + MAGIC.len()
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
    pub fn get_entry_by_key<Q>(&self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.get_entry(k)
    }

    #[must_use]
    pub fn get_entry_by_value<Q>(&self, v: &Q) -> Option<(K, V)>
    where
        V: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.converse_relation.get_entry(v).map(|(v, k)| (k, v))
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

    pub fn insert(&mut self, k: K, v: V) -> Result<(Option<K>, Option<V>), Error> {
        let maybe_old_val = self.direct_relation.get(&k);
        let maybe_old_key = self.converse_relation.get(&v);

        self.direct_relation.insert(k.clone(), v.clone())?;
        self.converse_relation
            .insert(v, k)
            .expect("insertion in second relation failed, the container is now corrupted");

        if let Some(ref old_key) = maybe_old_key {
            self.direct_relation.delete(old_key);
        }
        if let Some(ref old_val) = maybe_old_val {
            self.converse_relation.delete(old_val);
        }

        debug_assert_eq!(
            self.direct_relation.free_nodes_left(),
            self.converse_relation.free_nodes_left()
        );

        Ok((maybe_old_key, maybe_old_val))
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
        debug_assert_eq!(
            self.direct_relation.free_nodes_left(),
            self.converse_relation.free_nodes_left()
        );
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
        debug_assert_eq!(
            self.direct_relation.free_nodes_left(),
            self.converse_relation.free_nodes_left()
        );
        key
    }

    pub fn remove_entry_by_key<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.direct_relation.remove(key).map(|val| {
            let key = self.converse_relation.remove(&val);
            debug_assert_eq!(
                self.direct_relation.free_nodes_left(),
                self.converse_relation.free_nodes_left()
            );
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
            debug_assert_eq!(
                self.direct_relation.free_nodes_left(),
                self.converse_relation.free_nodes_left()
            );
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

        let container = OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        assert!(container.is_empty());

        unsafe {
            OneToOne::<u32, u32, 4, 4>::from_slice(&mut slice).unwrap();
        }
    }

    #[test]
    fn add_value() {
        let mut slice = vec![0; one_to_one_size(4, 4, 100)];

        let mut container = OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        assert_eq!(container.insert(1, 6).unwrap(), (None, None));
        assert_eq!(container.insert(2, 7).unwrap(), (None, None));
        assert_eq!(container.insert(3, 8).unwrap(), (None, None));
        assert_eq!(container.insert(4, 9).unwrap(), (None, None));
        assert_eq!(container.insert(5, 10).unwrap(), (None, None));

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

    #[test]
    fn add_string_values() {
        let mut slice = vec![0; one_to_one_size(10, 10, 10)];

        let mut container = OneToOne::<String, String, 10, 10>::init_slice(&mut slice).unwrap();

        assert_eq!(
            container
                .insert("one".to_string(), "two".to_string())
                .unwrap(),
            (None, None)
        );
        assert_eq!(
            container
                .insert("three".to_string(), "four".to_string())
                .unwrap(),
            (None, None)
        );
        assert_eq!(
            container
                .insert("five".to_string(), "six".to_string())
                .unwrap(),
            (None, None)
        );
        assert_eq!(
            container
                .insert("seven".to_string(), "eight".to_string())
                .unwrap(),
            (None, None)
        );

        // Failing new values
        assert_eq!(
            container.insert("too long key".to_string(), "smol".to_string()),
            Err(Error::KeySerializationError)
        );
        assert_eq!(
            container.insert("smol".to_string(), "too long value".to_string()),
            Err(Error::ValueSerializationError)
        );

        // By key
        assert_eq!(container.get_value("one"), Some("two".to_string()));
        assert_eq!(container.get_value("three"), Some("four".to_string()));
        assert_eq!(container.get_value("five"), Some("six".to_string()));
        assert_eq!(container.get_value("seven"), Some("eight".to_string()));

        assert_eq!(container.get_value("too long key"), None);
        assert_eq!(container.get_value("smol"), None);

        // By value
        assert_eq!(container.get_key("two"), Some("one".to_string()));
        assert_eq!(container.get_key("four"), Some("three".to_string()));
        assert_eq!(container.get_key("six"), Some("five".to_string()));
        assert_eq!(container.get_key("eight"), Some("seven".to_string()));

        assert_eq!(container.get_key("smol"), None);
        assert_eq!(container.get_key("too long value"), None);

        // Failing replacements for old values
        assert_eq!(
            container.insert("too long key".to_string(), "two".to_string()),
            Err(Error::KeySerializationError)
        );
        assert_eq!(
            container.insert("three".to_string(), "too long value".to_string()),
            Err(Error::ValueSerializationError)
        );

        // By key
        assert_eq!(container.get_value("one"), Some("two".to_string()));
        assert_eq!(container.get_value("three"), Some("four".to_string()));
        assert_eq!(container.get_value("five"), Some("six".to_string()));
        assert_eq!(container.get_value("seven"), Some("eight".to_string()));

        assert_eq!(container.get_value("too long key"), None);
        assert_eq!(container.get_value("smol"), None);

        // By value
        assert_eq!(container.get_key("two"), Some("one".to_string()));
        assert_eq!(container.get_key("four"), Some("three".to_string()));
        assert_eq!(container.get_key("six"), Some("five".to_string()));
        assert_eq!(container.get_key("eight"), Some("seven".to_string()));

        assert_eq!(container.get_key("smol"), None);
        assert_eq!(container.get_key("too long value"), None);

        // Ok replacements for old values
        assert_eq!(
            container
                .insert("five".to_string(), "nine".to_string())
                .unwrap(),
            (None, Some("six".to_string()))
        );
        assert_eq!(
            container
                .insert("seven".to_string(), "ten".to_string())
                .unwrap(),
            (None, Some("eight".to_string()))
        );

        // By key
        assert_eq!(container.get_value("one"), Some("two".to_string()));
        assert_eq!(container.get_value("three"), Some("four".to_string()));
        assert_eq!(container.get_value("five"), Some("nine".to_string()));
        assert_eq!(container.get_value("seven"), Some("ten".to_string()));

        assert_eq!(container.get_value("too long key"), None);
        assert_eq!(container.get_value("smol"), None);

        // By value
        assert_eq!(container.get_key("two"), Some("one".to_string()));
        assert_eq!(container.get_key("four"), Some("three".to_string()));
        assert_eq!(container.get_key("nine"), Some("five".to_string()));
        assert_eq!(container.get_key("ten"), Some("seven".to_string()));

        assert_eq!(container.get_key("smol"), None);
        assert_eq!(container.get_key("too long value"), None);

        // Ok replacement for old pair
        assert_eq!(
            container
                .insert("one".to_string(), "nine".to_string())
                .unwrap(),
            (Some("five".to_string()), Some("two".to_string()))
        );

        // By key
        assert_eq!(container.get_value("one"), Some("nine".to_string()));
        assert_eq!(container.get_value("three"), Some("four".to_string()));
        assert_eq!(container.get_value("five"), None);
        assert_eq!(container.get_value("seven"), Some("ten".to_string()));

        assert_eq!(container.get_value("too long key"), None);
        assert_eq!(container.get_value("smol"), None);

        // By value
        assert_eq!(container.get_key("two"), None);
        assert_eq!(container.get_key("four"), Some("three".to_string()));
        assert_eq!(container.get_key("nine"), Some("one".to_string()));
        assert_eq!(container.get_key("ten"), Some("seven".to_string()));

        assert_eq!(container.get_key("smol"), None);
        assert_eq!(container.get_key("too long value"), None);
    }

    #[test]
    fn remove_value() {
        let mut slice = vec![0; one_to_one_size(4, 4, 100)];

        let mut container = OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        assert_eq!(container.insert(1, 6).unwrap(), (None, None));
        assert_eq!(container.insert(2, 7).unwrap(), (None, None));
        assert_eq!(container.insert(3, 8).unwrap(), (None, None));
        assert_eq!(container.insert(4, 9).unwrap(), (None, None));
        assert_eq!(container.insert(5, 10).unwrap(), (None, None));

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

        assert_eq!(container.remove_by_key(&1), Some(6));
        assert_eq!(container.remove_by_key(&2), Some(7));
        assert_eq!(container.remove_by_value(&8), Some(3));
        assert_eq!(container.remove_by_value(&9), Some(4));
        assert_eq!(container.remove_by_value(&10), Some(5));

        assert_eq!(container.get_key(&6), None);
        assert_eq!(container.get_key(&7), None);
        assert_eq!(container.get_value(&3), None);
        assert_eq!(container.get_value(&4), None);
        assert_eq!(container.get_value(&5), None);

        assert!(container.is_empty());
    }

    #[test]
    fn remove_entry() {
        let mut slice = vec![0; one_to_one_size(4, 4, 100)];

        let mut container = OneToOne::<u32, u32, 4, 4>::init_slice(&mut slice).unwrap();

        assert_eq!(container.insert(1, 6).unwrap(), (None, None));
        assert_eq!(container.insert(2, 7).unwrap(), (None, None));
        assert_eq!(container.insert(3, 8).unwrap(), (None, None));
        assert_eq!(container.insert(4, 9).unwrap(), (None, None));
        assert_eq!(container.insert(5, 10).unwrap(), (None, None));

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

        assert_eq!(container.remove_entry_by_key(&1), Some((1, 6)));
        assert_eq!(container.remove_entry_by_key(&2), Some((2, 7)));
        assert_eq!(container.remove_entry_by_value(&8), Some((3, 8)));
        assert_eq!(container.remove_entry_by_value(&9), Some((4, 9)));
        assert_eq!(container.remove_entry_by_value(&10), Some((5, 10)));

        assert!(container.is_empty());
    }
}
