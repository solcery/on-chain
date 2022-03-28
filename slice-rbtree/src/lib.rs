use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast_mut, cast_slice_mut};
use std::borrow::Borrow;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;

mod header;
mod node;

use header::Header;
use node::Node;

#[derive(Debug)]
pub struct RBtree<'a, K, V, const KSIZE: usize, const VSIZE: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
{
    header: &'a mut Header,
    nodes: &'a mut [Node<KSIZE, VSIZE>],
    _phantom_key: PhantomData<K>,
    _phantom_value: PhantomData<V>,
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> RBtree<'a, K, V, KSIZE, VSIZE>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
{
    pub fn init_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() <= mem::size_of::<Header>() {
            return Err(Error::TooSmall);
        }

        let (header, nodes) = slice.split_at_mut(mem::size_of::<Header>());

        if nodes.len() == 0 {
            return Err(Error::TooSmall);
        }

        if nodes.len() % mem::size_of::<Node<KSIZE, VSIZE>>() != 0 {
            return Err(Error::WrongNodePoolSize);
        }
        let nodes: &mut [Node<KSIZE, VSIZE>] = cast_slice_mut(nodes);
        let header: &mut [[u8; mem::size_of::<Header>()]] = cast_slice_mut(header);
        let header: &mut Header = cast_mut(&mut header[0]);

        unsafe {
            // Allocator initialization
            nodes[0].set_parent(None);

            for (i, node) in nodes.iter_mut().enumerate().skip(1) {
                node.set_parent(Some((i - 1) as u32));
            }

            header.fill(
                KSIZE as u32,
                VSIZE as u32,
                nodes.len() as u32,
                None,
                Some((nodes.len() - 1) as u32),
            );
        }
        Ok(Self {
            header,
            nodes,
            _phantom_key: PhantomData::<K>,
            _phantom_value: PhantomData::<V>,
        })
    }

    pub unsafe fn from_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() <= mem::size_of::<Header>() {
            return Err(Error::TooSmall);
        }

        let (header, nodes) = slice.split_at_mut(mem::size_of::<Header>());

        if nodes.len() == 0 {
            return Err(Error::TooSmall);
        }

        if nodes.len() % mem::size_of::<Node<KSIZE, VSIZE>>() != 0 {
            return Err(Error::WrongNodePoolSize);
        }

        let nodes: &mut [Node<KSIZE, VSIZE>] = cast_slice_mut(nodes);
        let header: &mut [[u8; mem::size_of::<Header>()]] = cast_slice_mut(header);
        let header: &mut Header = cast_mut(&mut header[0]);

        if header.k_size() as usize != KSIZE {
            return Err(Error::WrongKeySize);
        }

        if header.v_size() as usize != VSIZE {
            return Err(Error::WrongValueSize);
        }

        Ok(Self {
            header,
            nodes,
            _phantom_key: PhantomData::<K>,
            _phantom_value: PhantomData::<V>,
        })
    }

    pub fn len(&self) -> usize {
        self.size(self.header.root())
    }

    /// Deallocates a node
    ///
    /// # Safety
    ///
    /// This function does nothing but deallocation. It should be checked, that the node is
    /// completely unlinked from the tree.
    unsafe fn delete_node(&mut self, index: usize) {
        let allocator_head = self.header.head();
        let node_index = Some(index as u32);

        self.nodes[index].set_parent(allocator_head);
        self.header.set_head(node_index);
    }

    /// Allocates a node
    ///
    /// # Safety
    ///
    /// This function does nothing but allocation. The returned node (if present) is
    /// completely unlinked from the tree and is in the unknown state. The caller must fill the
    /// node with correct data.
    fn allocate_node(&mut self) -> Option<usize> {
        let allocator_head = self.header.head();
        match allocator_head {
            Some(index) => {
                let new_head = self.nodes[index as usize].parent();
                unsafe {
                    self.header.set_head(new_head);
                }
                Some(index as usize)
            }
            None => None,
        }
    }

    pub fn clear(&mut self) {
        unsafe {
            // Allocator reinitialization
            self.nodes[0].set_parent(None);

            for (i, node) in self.nodes.iter_mut().enumerate().skip(1) {
                node.set_parent(Some((i - 1) as u32));
            }

            self.header.set_root(None);
            self.header.set_head(Some((self.nodes.len() - 1) as u32));
        }
    }

    pub fn contains_key<Q>(&self, k: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut maybe_id = self.header.root();
        while let Some(id) = maybe_id {
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).unwrap();
            match k.cmp(node_key.borrow()) {
                Ordering::Equal => {
                    return true;
                }
                Ordering::Less => maybe_id = node.left(),
                Ordering::Greater => maybe_id = node.right(),
            }
        }
        false
    }

    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut maybe_id = self.header.root();
        while let Some(id) = maybe_id {
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).unwrap();
            match k.cmp(node_key.borrow()) {
                Ordering::Equal => {
                    let node_value = V::deserialize(&mut node.value.as_slice()).unwrap();
                    return Some((node_key, node_value));
                }
                Ordering::Less => maybe_id = node.left(),
                Ordering::Greater => maybe_id = node.right(),
            }
        }
        None
    }

    pub fn get<Q>(&self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_value(k).map(|(_, v)| v)
    }

    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, Error> {
        let result = self.put(self.header.root(), None, key, value);
        match result {
            Ok((id, old_val)) => {
                unsafe {
                    self.header.set_root(Some(id));
                    self.nodes[id as usize].set_is_red(false);
                }
                return Ok(old_val);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        !self.header.root().is_some()
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.remove_entry(key).map(|(_, v)| v)
    }

    pub fn remove_entry<Q>(&mut self, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unimplemented!();
    }

    /// Deletes entry without deserializing the value.
    ///
    /// Return `true` if there was a value with the given `key`.
    pub fn delete<Q>(&mut self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        unimplemented!();
    }

    fn size(&self, maybe_id: Option<u32>) -> usize {
        if let Some(id) = maybe_id {
            self.nodes[id as usize].size() as usize
        } else {
            0
        }
    }

    fn put(
        &mut self,
        maybe_id: Option<u32>,
        parent: Option<u32>,
        key: K,
        value: V,
    ) -> Result<(u32, Option<V>), Error> {
        if let Some(mut id) = maybe_id {
            let old_val;
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).unwrap();
            match key.cmp(node_key.borrow()) {
                Ordering::Less => {
                    let left_result =
                        self.put(self.nodes[id as usize].left(), Some(id), key, value);
                    match left_result {
                        Ok((id, val)) => {
                            old_val = val;
                            unsafe {
                                self.nodes[id as usize].set_left(Some(id));
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ordering::Greater => {
                    let right_result =
                        self.put(self.nodes[id as usize].right(), Some(id), key, value);
                    match right_result {
                        Ok((id, val)) => {
                            old_val = val;
                            unsafe {
                                self.nodes[id as usize].set_right(Some(id));
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ordering::Equal => {
                    old_val = V::deserialize(&mut self.nodes[id as usize].value.as_slice()).ok();
                    let serialization_result =
                        value.serialize(&mut self.nodes[id as usize].value.as_mut_slice());
                    if serialization_result.is_err() {
                        return Err(Error::ValueSerializationError);
                    }
                }
            }
            unsafe {
                self.nodes[id as usize].set_size(
                    (self.size(self.nodes[id as usize].left())
                        + self.size(self.nodes[id as usize].right())
                        + 1) as u32,
                );

                if self.is_red(self.nodes[id as usize].right())
                    && !self.is_red(self.nodes[id as usize].left())
                {
                    id = self.rotate_left(id);
                }

                if self.is_red(self.nodes[id as usize].left())
                    && self
                        .is_red(self.nodes[self.nodes[id as usize].left().unwrap() as usize].left())
                {
                    id = self.rotate_right(id);
                }

                if self.is_red(self.nodes[id as usize].right())
                    && self.is_red(self.nodes[id as usize].left())
                {
                    let left_id = self.nodes[id as usize].left().unwrap() as usize;
                    let right_id = self.nodes[id as usize].right().unwrap() as usize;

                    self.nodes[left_id].set_is_red(!self.nodes[left_id].is_red());
                    self.nodes[right_id].set_is_red(!self.nodes[right_id].is_red());
                    self.nodes[id as usize].set_is_red(!self.nodes[id as usize].is_red());
                }
            }

            Ok((id, old_val))
        } else {
            let new_id = match self.allocate_node() {
                Some(id) => id,
                None => return Err(Error::NoNodesLeft),
            };
            let new_node = &mut self.nodes[new_id];

            if value.serialize(&mut new_node.value.as_mut_slice()).is_err() {
                unsafe {
                    // SAFETY: We are deleting previously allocated empty node, so no invariants
                    // are changed.
                    self.delete_node(new_id);
                }
                return Err(Error::ValueSerializationError);
            }

            if key.serialize(&mut new_node.key.as_mut_slice()).is_err() {
                unsafe {
                    // SAFETY: We are deleting previously allocated empty node, so no invariants
                    // are changed.
                    self.delete_node(new_id);
                }
                return Err(Error::KeySerializationError);
            }
            unsafe {
                new_node.set_parent(parent);
            }

            Ok((new_id as u32, None))
        }
    }

    fn is_red(&self, maybe_id: Option<u32>) -> bool {
        match maybe_id {
            Some(id) => self.nodes[id as usize].is_red(),
            None => false,
        }
    }

    unsafe fn rotate_left(&mut self, h: u32) -> u32 {
        let x = self.nodes[h as usize].right().unwrap();

        self.nodes[h as usize].set_right(self.nodes[x as usize].left());
        self.nodes[x as usize].set_left(Some(h));
        self.nodes[x as usize].set_is_red(self.nodes[h as usize].is_red());
        self.nodes[h as usize].set_is_red(true);

        // fix parents
        self.nodes[x as usize].set_parent(self.nodes[h as usize].parent());
        self.nodes[h as usize].set_parent(Some(x));
        if let Some(right) = self.nodes[h as usize].right() {
            self.nodes[right as usize].set_parent(Some(h));
        }

        // fix size
        self.nodes[x as usize].set_size(self.nodes[h as usize].size());
        self.nodes[h as usize].set_size(
            (self.size(self.nodes[h as usize].left())
                + self.size(self.nodes[h as usize].right())
                + 1) as u32,
        );

        x
    }

    unsafe fn rotate_right(&mut self, h: u32) -> u32 {
        let x = self.nodes[h as usize].left().unwrap();

        self.nodes[h as usize].set_left(self.nodes[x as usize].right());
        self.nodes[x as usize].set_right(Some(h));
        self.nodes[x as usize].set_is_red(self.nodes[h as usize].is_red());
        self.nodes[h as usize].set_is_red(true);

        // fix parents
        self.nodes[x as usize].set_parent(self.nodes[h as usize].parent());
        self.nodes[h as usize].set_parent(Some(x));
        if let Some(left) = self.nodes[h as usize].left() {
            self.nodes[left as usize].set_parent(Some(h));
        }

        // fix size
        self.nodes[x as usize].set_size(self.nodes[h as usize].size());
        self.nodes[h as usize].set_size(
            (self.size(self.nodes[h as usize].right())
                + self.size(self.nodes[h as usize].left())
                + 1) as u32,
        );

        x
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    TooSmall,
    WrongNodePoolSize,
    NoNodesLeft,
    ValueSerializationError,
    KeySerializationError,
    WrongKeySize,
    WrongValueSize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn create_vec(k_size: usize, v_size: usize, num_entries: usize) -> Vec<u8> {
        let len = mem::size_of::<Header>()
            + (mem::size_of::<Node<0, 0>>() + k_size + v_size) * num_entries;
        vec![0; len]
    }

    #[test]
    #[ignore]
    fn init() {
        let mut vec = create_vec(4, 4, 5);

        let mut tree = RBtree::<i32, u32, 4, 4>::init_slice(vec.as_mut_slice()).unwrap();
        assert!(tree.is_empty());

        assert_eq!(tree.insert(12, 32), Ok(None));
        dbg!(&tree);
        assert_eq!(tree.get(&12), Some(32));

        assert_eq!(tree.insert(32, 44), Ok(None));
        dbg!(&tree);
        assert_eq!(tree.get(&32), Some(44));

        assert_eq!(tree.insert(123, 321), Ok(None));
        assert_eq!(tree.get(&123), Some(321));

        assert_eq!(tree.insert(123, 322), Ok(Some(321)));
        assert_eq!(tree.get(&123), Some(322));

        assert_eq!(tree.insert(14, 32), Ok(None));
        assert_eq!(tree.get(&14), Some(32));

        assert_eq!(tree.insert(1, 2), Ok(None));
        assert_eq!(tree.insert(1, 4), Ok(Some(4)));
        assert_eq!(tree.insert(3, 4), Err(Error::NoNodesLeft));

        assert_eq!(tree.len(), 5);
    }
}
