use borsh::{BorshDeserialize, BorshSerialize};

use std::borrow::Borrow;

use slice_rbtree::{Error, KeysIterator, RBTree};

pub const MAGIC: &[u8; 18] = b"OneToOne container";

#[derive(Debug)]
// TODO: implement a proper Debug
pub struct OneToOne<'a, K, V, const KSIZE: usize, const VSIZE: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize + Clone,
    V: Ord + BorshDeserialize + BorshSerialize + Clone,
{
    direct_relation: RBTree<'a, K, V, KSIZE, VSIZE>,
    converse_relation: RBTree<'a, V, K, VSIZE, KSIZE>,
    magic: &'a [u8],
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
