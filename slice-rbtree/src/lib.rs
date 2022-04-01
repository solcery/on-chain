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
    unsafe fn deallocate_node(&mut self, index: usize) {
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
        self.get_key_index(k).is_some()
    }

    pub fn get_key_value<Q>(&self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(k).map(|id| {
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).unwrap();
            let node_value = V::deserialize(&mut node.value.as_slice()).unwrap();
            (node_key, node_value)
        })
    }

    pub fn get<Q>(&self, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(k).map(|id| {
            let node = &self.nodes[id as usize];
            let node_value = V::deserialize(&mut node.value.as_slice()).unwrap();
            node_value
        })
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
        match self.get_key_index(key) {
            Some(id) => unsafe { self.delete_node(id) },
            None => None,
        }
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
                        Ok((child_id, val)) => {
                            old_val = val;
                            unsafe {
                                self.nodes[id as usize].set_left(Some(child_id));
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ordering::Greater => {
                    let right_result =
                        self.put(self.nodes[id as usize].right(), Some(id), key, value);
                    match right_result {
                        Ok((child_id, val)) => {
                            old_val = val;
                            unsafe {
                                self.nodes[id as usize].set_right(Some(child_id));
                            }
                        }
                        Err(e) => return Err(e),
                    }
                }
                Ordering::Equal => {
                    old_val = V::deserialize(&mut self.nodes[id as usize].value.as_slice()).ok();
                    // This is needed to check if the value fits in the slice
                    // Otherwise we can invalidate data in the node
                    let mut serialization_container = [0; VSIZE];
                    let serialization_result =
                        value.serialize(&mut serialization_container.as_mut_slice());

                    match serialization_result {
                        Ok(()) => self.nodes[id as usize]
                            .value
                            .copy_from_slice(&serialization_container),
                        Err(_) => return Err(Error::ValueSerializationError),
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

                let left_subnode = match self.nodes[id as usize].left() {
                    Some(sub_id) => self.nodes[sub_id as usize].left(),
                    None => None,
                };

                if self.is_red(self.nodes[id as usize].left()) && self.is_red(left_subnode) {
                    id = self.rotate_right(id);
                }

                if self.is_red(self.nodes[id as usize].right())
                    && self.is_red(self.nodes[id as usize].left())
                {
                    // If nodes are red, they are not Option::None, so unwrap will never fail
                    let left_id = self.nodes[id as usize].left().unwrap() as usize;
                    let right_id = self.nodes[id as usize].right().unwrap() as usize;

                    // Color swap
                    self.nodes[left_id].set_is_red(false);
                    self.nodes[right_id].set_is_red(false);
                    self.nodes[id as usize].set_is_red(true);
                }
            }

            Ok((id, old_val))
        } else {
            let new_id = match self.allocate_node() {
                Some(id) => id,
                None => return Err(Error::NoNodesLeft),
            };
            let new_node = &mut self.nodes[new_id];

            unsafe {
                new_node.init_node(parent);
            }

            // Here it is ok to write directly to slice, because in case of error the node
            // will be deallocated anyway,
            if value.serialize(&mut new_node.value.as_mut_slice()).is_err() {
                unsafe {
                    // SAFETY: We are deleting previously allocated empty node, so no invariants
                    // are changed.
                    self.deallocate_node(new_id);
                }
                return Err(Error::ValueSerializationError);
            }

            if key.serialize(&mut new_node.key.as_mut_slice()).is_err() {
                unsafe {
                    self.deallocate_node(new_id);
                }
                return Err(Error::KeySerializationError);
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

    fn get_key_index<Q>(&self, k: &Q) -> Option<usize>
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
                    return Some(id as usize);
                }
                Ordering::Less => maybe_id = node.left(),
                Ordering::Greater => maybe_id = node.right(),
            }
        }
        None
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

    unsafe fn delete_node<Q>(&mut self, mut id: usize) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if self.nodes[id].left().is_some() && self.nodes[id].right().is_some() {
            id = self.swap_max_left(id);
        }

        match (self.nodes[id].left(), self.nodes[id].right()) {
            (Some(left), Some(right)) => {
                // Ureachable due to previously checked properties
                unreachable!("swap_max_left() returned a node with two children")
            }
            (Some(left), None) => {
                debug_assert!(self.nodes[left as usize].is_red());
                self.swap_nodes(id, left as usize);

                let key = K::deserialize(&mut self.nodes[id].key.as_slice()).unwrap();
                let value = V::deserialize(&mut self.nodes[id].value.as_slice()).unwrap();

                self.deallocate_node(id);

                return Some((key, value));
            }
            (None, Some(right)) => {
                debug_assert!(self.nodes[right as usize].is_red());
                self.swap_nodes(id, right as usize);

                let key = K::deserialize(&mut self.nodes[id].key.as_slice()).unwrap();
                let value = V::deserialize(&mut self.nodes[id].value.as_slice()).unwrap();

                self.deallocate_node(id);

                return Some((key, value));
            }
            (None, None) => {
                if self.nodes[id].is_red() {
                    let parent_id = self.nodes[id].parent().unwrap();
                    let parent_node = &mut self.nodes[parent_id as usize];

                    if parent_node.left() == Some(id as u32) {
                        parent_node.set_left(None);
                    } else {
                        debug_assert_eq!(parent_node.right(), Some(id as u32));

                        parent_node.set_right(None);
                    }

                    let key = K::deserialize(&mut self.nodes[id].key.as_slice()).unwrap();
                    let value = V::deserialize(&mut self.nodes[id].value.as_slice()).unwrap();

                    self.deallocate_node(id);

                    return Some((key, value));
                } else {
                    let key = K::deserialize(&mut self.nodes[id].key.as_slice()).unwrap();
                    let value = V::deserialize(&mut self.nodes[id].value.as_slice()).unwrap();

                    self.deallocate_node(id);

                    if let Some(parent_id) = self.nodes[id].parent() {
                        self.balance_subtree(parent_id as usize);
                    }

                    return Some((key, value));
                }
            }
        }
    }

    unsafe fn swap_max_left(&mut self, id: usize) -> usize {
        let mut max_id = self.nodes[id].left().unwrap() as usize;
        while let Some(maybe_max) = self.nodes[max_id].right() {
            max_id = maybe_max as usize;
        }

        debug_assert_ne!(id, max_id);
        self.swap_nodes(id, max_id);
        max_id
    }

    unsafe fn swap_nodes(&mut self, a: usize, b: usize) {
        let tmp_key = self.nodes[a].key;
        self.nodes[a].key = self.nodes[b].key;
        self.nodes[b].key = tmp_key;

        let tmp_value = self.nodes[a].value;
        self.nodes[a].value = self.nodes[b].value;
        self.nodes[b].value = tmp_value;
    }

    // id of the parent node of subtree to be balanced
    unsafe fn balance_subtree(&mut self, id: usize) {
        let left_child = self.nodes[id].left();
        let right_child = self.nodes[id].right();
        debug_assert_ne!(self.size(left_child), self.size(right_child));

        todo!("Ч0");
    }
}

#[cfg(test)]
impl<'a, K, V, const KSIZE: usize, const VSIZE: usize> RBtree<'a, K, V, KSIZE, VSIZE>
where
    K: Eq + Ord + BorshDeserialize + BorshSerialize,
    V: Eq + BorshDeserialize + BorshSerialize,
{
    fn is_balanced(&self) -> bool {
        let mut black = 0;
        let mut node = self.header.root();
        while let Some(id) = node {
            if !self.nodes[id as usize].is_red() {
                black += 1;
            }
            node = self.nodes[id as usize].left();
        }
        self.node_balanced(self.header.root(), black)
    }

    fn node_balanced(&self, maybe_id: Option<u32>, black: u32) -> bool {
        if let Some(id) = maybe_id {
            let id = id as usize;
            if self.nodes[id].is_red() {
                let is_left_balanced = self.node_balanced(self.nodes[id].left(), black);
                let is_right_balanced = self.node_balanced(self.nodes[id].right(), black);

                is_left_balanced && is_right_balanced
            } else {
                let is_left_balanced = self.node_balanced(self.nodes[id].left(), black - 1);
                let is_right_balanced = self.node_balanced(self.nodes[id].right(), black - 1);

                is_left_balanced && is_right_balanced
            }
        } else {
            black == 0
        }
    }

    unsafe fn set_node(&mut self, id: usize, node: &Node<KSIZE, VSIZE>) {
        self.nodes[id] = *node;
    }

    unsafe fn set_root(&mut self, id: usize, root: Option<u32>) {
        self.header.set_root(root);
    }

    unsafe fn set_head(&mut self, id: usize, head: Option<u32>) {
        self.header.set_head(head);
    }

    fn struct_eq(&self, other: &Self) -> bool {
        self.node_eq(self.header.root(), other.header.root())
    }

    fn node_eq(&self, a: Option<u32>, b: Option<u32>) -> bool {
        match (a, b) {
            (Some(self_id), Some(other_id)) => {
                let self_id = self_id as usize;
                let other_id = other_id as usize;

                if self.nodes[self_id].is_red() ^ self.nodes[self_id].is_red() {
                    return false;
                }

                let self_key = K::deserialize(&mut self.nodes[self_id].key.as_slice()).unwrap();
                let other_key = K::deserialize(&mut self.nodes[other_id].key.as_slice()).unwrap();

                if self_key != other_key {
                    return false;
                }

                let self_left = self.nodes[self_id].left();
                let other_left = self.nodes[other_id].left();

                let self_right = self.nodes[self_id].right();
                let other_right = self.nodes[other_id].right();

                self.node_eq(self_left, other_left) && self.node_eq(self_right, other_right)
            }
            (None, None) => true,
            _ => false,
        }
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

    #[test]
    fn init() {
        let mut vec = create_vec(4, 4, 5);

        let mut tree = RBtree::<i32, u32, 4, 4>::init_slice(vec.as_mut_slice()).unwrap();
        assert!(tree.is_empty());

        assert_eq!(tree.insert(12, 32), Ok(None));
        assert_eq!(tree.get(&12), Some(32));
        assert_eq!(tree.len(), 1);

        assert_eq!(tree.insert(32, 44), Ok(None));
        assert_eq!(tree.get(&32), Some(44));
        assert_eq!(tree.len(), 2);

        assert_eq!(tree.insert(123, 321), Ok(None));
        assert_eq!(tree.get(&123), Some(321));
        assert_eq!(tree.len(), 3);

        assert_eq!(tree.insert(123, 322), Ok(Some(321)));
        assert_eq!(tree.get(&123), Some(322));
        assert_eq!(tree.len(), 3);

        assert_eq!(tree.insert(14, 32), Ok(None));
        assert_eq!(tree.get(&14), Some(32));
        assert_eq!(tree.len(), 4);

        assert_eq!(tree.insert(1, 2), Ok(None));
        assert_eq!(tree.insert(1, 4), Ok(Some(2)));
        assert_eq!(tree.insert(3, 4), Err(Error::NoNodesLeft));

        assert_eq!(tree.get(&15), None);

        assert_eq!(tree.len(), 5);
    }

    #[test]
    fn swap_nodes() {
        let mut vec = create_vec(4, 4, 6);

        let mut tree = RBtree::<i32, u32, 4, 4>::init_slice(vec.as_mut_slice()).unwrap();
        // Initial structure
        //          parent
        //           /
        // black-> swap1
        //        /   \
        //red-> swap2 node1 <-red
        //      /
        //  node2            <-black
        unsafe {
            let parent = Node::from_raw_parts(
                // 0
                u32::to_be_bytes(1),
                u32::to_be_bytes(4),
                3,
                Some(1),
                None,
                None,
                false,
            );

            let swap1 = Node::from_raw_parts(
                // 1
                u32::to_be_bytes(2),
                u32::to_be_bytes(5),
                2,
                Some(2),
                Some(3),
                Some(0),
                false,
            );

            let swap2 = Node::from_raw_parts(
                // 2
                u32::to_be_bytes(3),
                u32::to_be_bytes(6),
                2,
                Some(4),
                None,
                Some(1),
                true,
            );

            let node1 = Node::from_raw_parts(
                // 3
                u32::to_be_bytes(7),
                u32::to_be_bytes(9),
                2,
                None,
                None,
                Some(1),
                true,
            );

            let node2 = Node::from_raw_parts(
                // 4
                u32::to_be_bytes(8),
                u32::to_be_bytes(8),
                2,
                None,
                None,
                Some(2),
                false,
            );

            tree.set_node(0, &parent);
            tree.set_node(1, &swap1);
            tree.set_node(2, &swap2);
            tree.set_node(3, &node1);
            tree.set_node(4, &node2);
        }

        let mut expected_vec = create_vec(4, 4, 6);

        let mut expected_tree =
            RBtree::<i32, u32, 4, 4>::init_slice(expected_vec.as_mut_slice()).unwrap();
        // Final structure
        //          parent
        //           /
        // black-> swap2
        //        /   \
        //red-> swap1 node1 <-red
        //      /
        //  node2            <-black
        unsafe {
            let parent = Node::from_raw_parts(
                // 0
                u32::to_be_bytes(1),
                u32::to_be_bytes(4),
                3,
                Some(1),
                None,
                None,
                false,
            );

            let swap2 = Node::from_raw_parts(
                // 1
                u32::to_be_bytes(2),
                u32::to_be_bytes(5),
                2,
                Some(4),
                None,
                Some(1),
                true,
            );

            let swap1 = Node::from_raw_parts(
                // 2
                u32::to_be_bytes(3),
                u32::to_be_bytes(6),
                2,
                Some(2),
                Some(3),
                Some(0),
                false,
            );

            let node1 = Node::from_raw_parts(
                // 3
                u32::to_be_bytes(7),
                u32::to_be_bytes(9),
                2,
                None,
                None,
                Some(1),
                true,
            );

            let node2 = Node::from_raw_parts(
                // 4
                u32::to_be_bytes(8),
                u32::to_be_bytes(8),
                2,
                None,
                None,
                Some(2),
                false,
            );

            expected_tree.set_node(0, &parent);
            expected_tree.set_node(1, &swap2);
            expected_tree.set_node(2, &swap1);
            expected_tree.set_node(3, &node1);
            expected_tree.set_node(4, &node2);
        }

        assert!(tree.struct_eq(&expected_tree));
    }

    #[test]
    fn test_tree_strings() {
        let mut vec = create_vec(4, 10, 10);

        let mut tree = RBtree::<i32, String, 4, 10>::init_slice(vec.as_mut_slice()).unwrap();
        assert!(tree.is_empty());

        assert_eq!(tree.insert(12, "val".to_string()), Ok(None));
        assert_eq!(tree.insert(32, "44".to_string()), Ok(None));
        assert_eq!(tree.insert(123, "321".to_string()), Ok(None));
        assert_eq!(
            tree.insert(123, "321".to_string()),
            Ok(Some("321".to_string()))
        );
        assert_eq!(tree.insert(1, "2".to_string()), Ok(None));
        assert_eq!(tree.insert(14, "32".to_string()), Ok(None));
        assert_eq!(tree.insert(20, "41".to_string()), Ok(None));
        assert_eq!(tree.insert(6, "64".to_string()), Ok(None));
        assert_eq!(tree.insert(41, "22".to_string()), Ok(None));
        assert_eq!(tree.insert(122, "14".to_string()), Ok(None));
        assert_eq!(
            tree.insert(41, "99".to_string()),
            Ok(Some("22".to_string()))
        );
        assert_eq!(
            tree.insert(12, "very long value".to_string()),
            Err(Error::ValueSerializationError)
        );

        assert_eq!(tree.get(&41).unwrap(), "99".to_string());
        assert_eq!(tree.get(&12).unwrap(), "val".to_string());
        assert_eq!(tree.len(), 9);
    }

    #[test]
    fn test_tree_string_keys() {
        let mut vec = create_vec(10, 10, 10);

        let mut tree = RBtree::<String, String, 10, 10>::init_slice(vec.as_mut_slice()).unwrap();
        assert!(tree.is_empty());

        assert_eq!(tree.insert("12".to_string(), "val".to_string()), Ok(None));
        assert_eq!(tree.insert("32".to_string(), "44".to_string()), Ok(None));
        assert_eq!(tree.insert("123".to_string(), "321".to_string()), Ok(None));
        assert_eq!(
            tree.insert("123".to_string(), "321".to_string()),
            Ok(Some("321".to_string()))
        );
        assert_eq!(tree.insert("1".to_string(), "2".to_string()), Ok(None));
        assert_eq!(tree.insert("14".to_string(), "32".to_string()), Ok(None));
        assert_eq!(tree.insert("20".to_string(), "41".to_string()), Ok(None));
        assert_eq!(tree.insert("6".to_string(), "64".to_string()), Ok(None));
        assert_eq!(tree.insert("41".to_string(), "22".to_string()), Ok(None));
        assert_eq!(tree.insert("122".to_string(), "14".to_string()), Ok(None));
        assert_eq!(
            tree.insert("41".to_string(), "99".to_string()),
            Ok(Some("22".to_string()))
        );

        assert_eq!(
            tree.insert("12".to_string(), "very long value".to_string()),
            Err(Error::ValueSerializationError)
        );

        assert_eq!(
            tree.insert("very long key".to_string(), "1".to_string()),
            Err(Error::KeySerializationError)
        );

        assert_eq!(tree.get(&"41".to_string()).unwrap(), "99".to_string());
        assert_eq!(tree.get(&"12".to_string()).unwrap(), "val".to_string());
        assert_eq!(tree.len(), 9);
    }

    #[test]
    #[ignore]
    fn delete() {
        let mut vec = create_vec(4, 4, 5);

        let mut tree = RBtree::<i32, u32, 4, 4>::init_slice(vec.as_mut_slice()).unwrap();
        assert!(tree.is_empty());

        assert_eq!(tree.insert(12, 32), Ok(None));
        assert_eq!(tree.insert(32, 44), Ok(None));
        assert_eq!(tree.insert(123, 321), Ok(None));
        assert_eq!(tree.insert(14, 32), Ok(None));
        assert_eq!(tree.insert(1, 2), Ok(None));

        dbg!(&tree);

        assert_eq!(tree.len(), 5);

        assert_rm(12, &mut tree, 4);
        assert_rm(32, &mut tree, 3);
        assert_rm(123, &mut tree, 2);
        assert_rm(14, &mut tree, 1);
        assert_rm(1, &mut tree, 0);
        assert!(tree.is_empty());
    }

    fn create_vec(k_size: usize, v_size: usize, num_entries: usize) -> Vec<u8> {
        let len = mem::size_of::<Header>()
            + (mem::size_of::<Node<0, 0>>() + k_size + v_size) * num_entries;
        vec![0; len]
    }

    fn assert_rm<'a, K, V, const KSIZE: usize, const VSIZE: usize>(
        val: K,
        tree: &mut RBtree<'a, K, V, KSIZE, VSIZE>,
        size: usize,
    ) where
        K: Eq + Ord + BorshDeserialize + BorshSerialize,
        V: Eq + BorshDeserialize + BorshSerialize,
    {
        assert!(tree.is_balanced());
        assert!(tree.contains_key(&val));
        assert!(tree.remove_entry(&val).is_some());
        assert!(!tree.contains_key(&val));
        assert!(tree.is_balanced());
        //assert_eq!(tree.len(), size);
    }
}
