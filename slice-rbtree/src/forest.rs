use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast_mut, cast_slice_mut};
use std::borrow::Borrow;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;

mod header;
mod node;

pub(crate) use header::Header;
pub(crate) use node::Node;

use super::Error;

//#[derive(Debug)]
pub struct RBForest<'a, K, V, const KSIZE: usize, const VSIZE: usize, const MAX_ROOTS: usize>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    header: &'a mut Header<MAX_ROOTS>,
    nodes: &'a mut [Node<KSIZE, VSIZE>],
    _phantom_key: PhantomData<K>,
    _phantom_value: PhantomData<V>,
}

impl<'a, K, V, const KSIZE: usize, const VSIZE: usize, const MAX_ROOTS: usize>
    RBForest<'a, K, V, KSIZE, VSIZE, MAX_ROOTS>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    pub fn init_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() <= mem::size_of::<Header<MAX_ROOTS>>() {
            return Err(Error::TooSmall);
        }

        let (header, nodes) = slice.split_at_mut(mem::size_of::<Header<MAX_ROOTS>>());

        if nodes.is_empty() {
            return Err(Error::TooSmall);
        }

        if nodes.len() % mem::size_of::<Node<KSIZE, VSIZE>>() != 0 {
            return Err(Error::WrongNodePoolSize);
        }

        let nodes: &mut [Node<KSIZE, VSIZE>] = cast_slice_mut(nodes);
        let header: &mut [[u8; mem::size_of::<Header<MAX_ROOTS>>()]] = cast_slice_mut(header);
        let header: &mut Header<MAX_ROOTS> = cast_mut(&mut header[0]);

        unsafe {
            // Allocator initialization
            nodes[0].set_parent(None);

            for (i, node) in nodes.iter_mut().enumerate().skip(1) {
                node.set_parent(Some((i - 1) as u32));
            }

            header.fill(
                KSIZE as u16,
                VSIZE as u16,
                nodes.len() as u32,
                [None; MAX_ROOTS],
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

    #[must_use]
    pub fn expected_size(num_entries: usize) -> usize {
        mem::size_of::<Header<MAX_ROOTS>>() + mem::size_of::<Node<KSIZE, VSIZE>>() * num_entries
    }

    pub unsafe fn from_slice(slice: &'a mut [u8]) -> Result<Self, Error> {
        if slice.len() <= mem::size_of::<Header<MAX_ROOTS>>() {
            return Err(Error::TooSmall);
        }

        let (header, nodes) = slice.split_at_mut(mem::size_of::<Header<MAX_ROOTS>>());

        if nodes.is_empty() {
            return Err(Error::TooSmall);
        }

        if nodes.len() % mem::size_of::<Node<KSIZE, VSIZE>>() != 0 {
            return Err(Error::WrongNodePoolSize);
        }

        let nodes: &mut [Node<KSIZE, VSIZE>] = cast_slice_mut(nodes);
        let header: &mut [[u8; mem::size_of::<Header<MAX_ROOTS>>()]] = cast_slice_mut(header);
        let header: &mut Header<MAX_ROOTS> = cast_mut(&mut header[0]);

        if header.roots_num() as usize != MAX_ROOTS {
            return Err(Error::WrongRootsNumber);
        }

        if header.k_size() as usize != KSIZE {
            return Err(Error::WrongKeySize);
        }

        if header.v_size() as usize != VSIZE {
            return Err(Error::WrongValueSize);
        }

        if header.max_nodes() as usize != nodes.len() {
            return Err(Error::WrongNodePoolSize);
        }

        Ok(Self {
            header,
            nodes,
            _phantom_key: PhantomData::<K>,
            _phantom_value: PhantomData::<V>,
        })
    }

    #[must_use]
    pub fn len(&self, tree_id: usize) -> usize {
        self.size(self.header.root(tree_id))
    }

    pub fn clear(&mut self) {
        unsafe {
            // Allocator reinitialization
            self.nodes[0].set_parent(None);

            for (i, node) in self.nodes.iter_mut().enumerate().skip(1) {
                node.set_parent(Some((i - 1) as u32));
            }

            for tree_id in 0..MAX_ROOTS {
                self.header.set_root(tree_id, None);
            }
            self.header.set_head(Some((self.nodes.len() - 1) as u32));
        }
    }

    #[must_use]
    pub fn contains_key<Q>(&self, tree_id: usize, k: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, k).is_some()
    }

    #[must_use]
    pub fn get_key_value<Q>(&self, tree_id: usize, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, k).map(|id| {
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).expect("Key corrupted");
            let node_value = V::deserialize(&mut node.value.as_slice()).expect("Value corrupted");
            (node_key, node_value)
        })
    }

    #[must_use]
    pub fn get<Q>(&self, tree_id: usize, k: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, k).map(|id| {
            let node = &self.nodes[id as usize];
            let node_value = V::deserialize(&mut node.value.as_slice()).expect("Value corrupted");
            node_value
        })
    }

    pub fn insert(&mut self, tree_id: usize, key: K, value: V) -> Result<Option<V>, Error> {
        let result = self.put(tree_id, self.header.root(tree_id), None, key, value);
        match result {
            Ok((id, old_val)) => {
                unsafe {
                    self.header.set_root(tree_id, Some(id));
                    self.nodes[id as usize].set_is_red(false);
                }
                Ok(old_val)
            }
            Err(e) => Err(e),
        }
    }

    #[must_use]
    pub fn is_empty(&self, tree_id: usize) -> bool {
        self.header.root(tree_id).is_none()
    }

    pub fn remove<Q>(&mut self, tree_id: usize, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, key).map(|id| {
            let deallocated_node_id = unsafe { self.delete_node(tree_id, id) };

            let value = V::deserialize(&mut self.nodes[deallocated_node_id].value.as_slice())
                .expect("Value corrupted");
            value
        })
    }

    pub fn remove_entry<Q>(&mut self, tree_id: usize, key: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, key).map(|id| {
            let deallocated_node_id = unsafe { self.delete_node(tree_id, id) };

            let key = K::deserialize(&mut self.nodes[deallocated_node_id].key.as_slice())
                .expect("Key corrupted");
            let value = V::deserialize(&mut self.nodes[deallocated_node_id].value.as_slice())
                .expect("Value corrupted");
            (key, value)
        })
    }

    /// Deletes entry without deserializing the value.
    ///
    /// Return `true` if there was a value with the given `key`.
    pub fn delete<Q>(&mut self, tree_id: usize, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        self.get_key_index(tree_id, key)
            .map(|id| unsafe {
                self.delete_node(tree_id, id);
            })
            .is_some()
    }

    pub fn pairs<'b>(
        &'b self,
        tree_id: usize,
    ) -> PairsIterator<'b, 'a, K, V, KSIZE, VSIZE, MAX_ROOTS> {
        PairsIterator {
            next_node: self
                .header
                .root(tree_id)
                .map(|root_id| self.min(root_id as usize)),
            tree: self,
        }
    }

    pub fn keys<'b>(
        &'b self,
        tree_id: usize,
    ) -> KeysIterator<'b, 'a, K, V, KSIZE, VSIZE, MAX_ROOTS> {
        KeysIterator {
            next_node: self
                .header
                .root(tree_id)
                .map(|root_id| self.min(root_id as usize)),
            tree: self,
        }
    }

    pub fn values<'b>(
        &'b self,
        tree_id: usize,
    ) -> ValuesIterator<'b, 'a, K, V, KSIZE, VSIZE, MAX_ROOTS> {
        ValuesIterator {
            next_node: self
                .header
                .root(tree_id)
                .map(|root_id| self.min(root_id as usize)),
            tree: self,
        }
    }

    pub fn first_entry(&self, tree_id: usize) -> Option<(K, V)> {
        self.header.root(tree_id).map(|root_id| {
            let node = &self.nodes[self.min(root_id as usize)];
            let key = K::deserialize(&mut node.key.as_slice()).expect("Key corrupted");
            let value = V::deserialize(&mut node.value.as_slice()).expect("Value corrupted");
            (key, value)
        })
    }

    pub fn last_entry(&self, tree_id: usize) -> Option<(K, V)> {
        self.header.root(tree_id).map(|root_id| {
            let node = &self.nodes[self.max(root_id as usize)];
            let key = K::deserialize(&mut node.key.as_slice()).expect("Key corrupted");
            let value = V::deserialize(&mut node.value.as_slice()).expect("Value corrupted");
            (key, value)
        })
    }

    #[must_use]
    fn size(&self, maybe_id: Option<u32>) -> usize {
        if let Some(id) = maybe_id {
            let node = self.nodes[id as usize];
            self.size(node.left()) + self.size(node.right()) + 1
        } else {
            0
        }
    }

    fn put(
        &mut self,
        tree_id: usize,
        maybe_id: Option<u32>,
        parent: Option<u32>,
        key: K,
        value: V,
    ) -> Result<(u32, Option<V>), Error> {
        if let Some(mut id) = maybe_id {
            let old_val;
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).expect("Key corrupted");
            match key.cmp(node_key.borrow()) {
                Ordering::Less => {
                    let left_result = self.put(
                        tree_id,
                        self.nodes[id as usize].left(),
                        Some(id),
                        key,
                        value,
                    );
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
                    let right_result = self.put(
                        tree_id,
                        self.nodes[id as usize].right(),
                        Some(id),
                        key,
                        value,
                    );
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
                if self.is_red(self.nodes[id as usize].right())
                    && !self.is_red(self.nodes[id as usize].left())
                {
                    id = self.rotate_left(tree_id, id);
                }

                let left_subnode = match self.nodes[id as usize].left() {
                    Some(sub_id) => self.nodes[sub_id as usize].left(),
                    None => None,
                };

                if self.is_red(self.nodes[id as usize].left()) && self.is_red(left_subnode) {
                    id = self.rotate_right(tree_id, id);
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

    #[must_use]
    fn is_red(&self, maybe_id: Option<u32>) -> bool {
        match maybe_id {
            Some(id) => self.nodes[id as usize].is_red(),
            None => false,
        }
    }

    fn get_key_index<Q>(&self, tree_id: usize, k: &Q) -> Option<usize>
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        let mut maybe_id = self.header.root(tree_id);
        while let Some(id) = maybe_id {
            let node = &self.nodes[id as usize];
            let node_key = K::deserialize(&mut node.key.as_slice()).expect("Key corrupted");
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

    unsafe fn rotate_left(&mut self, tree_id: usize, h: u32) -> u32 {
        let x = self.nodes[h as usize]
            .right()
            .expect("RBTree invariants corrupted: rotate_left on subtree without right child");

        unsafe {
            self.nodes[h as usize].set_right(self.nodes[x as usize].left());
            self.nodes[x as usize].set_left(Some(h));
            self.nodes[x as usize].set_is_red(self.nodes[h as usize].is_red());
            self.nodes[h as usize].set_is_red(true);

            // fix parents
            if let Some(parent_id) = self.nodes[h as usize].parent() {
                let parent_node = &mut self.nodes[parent_id as usize];
                if parent_node.left() == Some(h) {
                    parent_node.set_left(Some(x));
                } else {
                    debug_assert_eq!(parent_node.right(), Some(h));

                    parent_node.set_right(Some(x));
                }
            } else {
                self.header.set_root(tree_id, Some(x));
            }
            self.nodes[x as usize].set_parent(self.nodes[h as usize].parent());
            self.nodes[h as usize].set_parent(Some(x));
            if let Some(right) = self.nodes[h as usize].right() {
                self.nodes[right as usize].set_parent(Some(h));
            }
        }

        x
    }

    unsafe fn rotate_right(&mut self, tree_id: usize, h: u32) -> u32 {
        let x = self.nodes[h as usize]
            .left()
            .expect("RBTree invariants corrupted: rotate_left on subtree without left child");

        unsafe {
            self.nodes[h as usize].set_left(self.nodes[x as usize].right());
            self.nodes[x as usize].set_right(Some(h));
            self.nodes[x as usize].set_is_red(self.nodes[h as usize].is_red());
            self.nodes[h as usize].set_is_red(true);

            // fix parents
            if let Some(parent_id) = self.nodes[h as usize].parent() {
                let parent_node = &mut self.nodes[parent_id as usize];
                if parent_node.left() == Some(h) {
                    parent_node.set_left(Some(x));
                } else {
                    debug_assert_eq!(parent_node.right(), Some(h));

                    parent_node.set_right(Some(x));
                }
            } else {
                self.header.set_root(tree_id, Some(x));
            }
            self.nodes[x as usize].set_parent(self.nodes[h as usize].parent());
            self.nodes[h as usize].set_parent(Some(x));
            if let Some(left) = self.nodes[h as usize].left() {
                self.nodes[left as usize].set_parent(Some(h));
            }
        }

        x
    }

    unsafe fn delete_node<Q>(&mut self, tree_id: usize, mut id: usize) -> usize
    where
        K: Borrow<Q> + Ord,
        Q: Ord + ?Sized,
    {
        if self.nodes[id].left().is_some() && self.nodes[id].right().is_some() {
            unsafe {
                id = self.swap_max_left(id);
            }
        }

        match (self.nodes[id].left(), self.nodes[id].right()) {
            (Some(_), Some(_)) => {
                unreachable!("swap_max_left() returned a node with two children");
            }
            (Some(left), None) => {
                let left_id = left as usize;
                // This node has to be black, its child has to be red
                debug_assert!(!self.nodes[id].is_red());
                debug_assert!(self.nodes[left_id].is_red());

                unsafe {
                    self.swap_nodes(id, left_id);

                    self.nodes[id].set_left(None);
                    self.deallocate_node(left_id);
                }

                left_id
            }
            (None, Some(right)) => {
                let right_id = right as usize;
                // This node has to be black, its child has to be red
                debug_assert!(!self.nodes[id].is_red());
                debug_assert!(self.nodes[right_id].is_red());

                unsafe {
                    self.swap_nodes(id, right_id);

                    self.nodes[id].set_right(None);

                    self.deallocate_node(right_id);
                }

                right_id
            }
            (None, None) => {
                if self.nodes[id].is_red() {
                    // Root node is always black, so if nodes[id] is red, it always has a parent
                    let parent_id = self.nodes[id].parent().unwrap();
                    let parent_node = &mut self.nodes[parent_id as usize];

                    unsafe {
                        if parent_node.left() == Some(id as u32) {
                            parent_node.set_left(None);
                        } else {
                            debug_assert_eq!(parent_node.right(), Some(id as u32));

                            parent_node.set_right(None);
                        }

                        self.deallocate_node(id);
                    }

                    id
                } else {
                    if let Some(parent_id) = self.nodes[id].parent() {
                        let parent_node = &mut self.nodes[parent_id as usize];
                        unsafe {
                            if parent_node.left() == Some(id as u32) {
                                parent_node.set_left(None);
                            } else {
                                debug_assert_eq!(parent_node.right(), Some(id as u32));

                                parent_node.set_right(None);
                            }

                            self.balance_subtree(tree_id, parent_id as usize);
                        }
                    } else {
                        unsafe {
                            self.header.set_root(tree_id, None);
                        }
                    }

                    unsafe {
                        self.deallocate_node(id);
                    }

                    id
                }
            }
        }
    }

    #[must_use]
    unsafe fn swap_max_left(&mut self, id: usize) -> usize {
        let mut max_id = self.nodes[id]
            .left()
            .expect("swap_max_left should only be called on nodes with two children")
            as usize;
        while let Some(maybe_max) = self.nodes[max_id].right() {
            max_id = maybe_max as usize;
        }

        debug_assert_ne!(id, max_id);
        unsafe {
            self.swap_nodes(id, max_id);
        }
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

    unsafe fn balance_subtree(&mut self, tree_id: usize, id: usize) {
        let left_child = self.nodes[id].left();
        let right_child = self.nodes[id].right();
        let left_depth = self.black_depth(left_child);
        let right_depth = self.black_depth(right_child);
        match left_depth.cmp(&right_depth) {
            Ordering::Greater => {
                // left_depth is greater than right_depth, so it is >= 1 and therefore left_child
                // always exists
                let left_id = left_child.unwrap() as usize;
                if self.nodes[id].is_red() {
                    debug_assert!(!self.nodes[left_id].is_red());
                    let left_grandchild = self.nodes[left_id].left();
                    let right_grandchild = self.nodes[left_id].right();
                    match (self.is_red(left_grandchild), self.is_red(right_grandchild)) {
                        (false, false) => unsafe {
                            self.nodes[id].set_is_red(false);
                            self.nodes[left_id].set_is_red(true);
                        },
                        (true, _) => unsafe {
                            self.rotate_right(tree_id, id as u32);

                            self.nodes[id].set_is_red(false);
                            self.nodes[left_id].set_is_red(true);
                            // left_grandchild is red, so it exists
                            self.nodes[left_grandchild.unwrap() as usize].set_is_red(false);
                        },
                        (false, true) => unsafe {
                            self.rotate_left(tree_id, left_id as u32);
                            self.rotate_right(tree_id, id as u32);
                            // right_grandchild is red, so it exists
                            self.nodes[right_grandchild.unwrap() as usize].set_is_red(false);
                        },
                    }
                } else if self.nodes[left_id].is_red() {
                    debug_assert!(!self.is_red(self.nodes[left_id].left()));
                    debug_assert!(!self.is_red(self.nodes[left_id].right()));
                    // left_depth is greater than right_depth, so it is >= 1
                    // left_child is red and does not affect black height
                    // therefore left and right grandchildren exists
                    let right_grandchild = self.nodes[left_id].right().unwrap() as usize;
                    let left_grandgrandchild = self.nodes[right_grandchild].left();
                    let right_grandgrandchild = self.nodes[right_grandchild].right();

                    match (
                        self.is_red(left_grandgrandchild),
                        self.is_red(right_grandgrandchild),
                    ) {
                        (false, false) => unsafe {
                            self.rotate_right(tree_id, id as u32);
                            self.nodes[id].set_is_red(false);
                            self.nodes[right_grandchild].set_is_red(true);
                        },
                        (true, _) => unsafe {
                            self.rotate_left(tree_id, left_id as u32);
                            self.rotate_right(tree_id, id as u32);
                            // left_grandgrandchild is red, so it always exists
                            self.nodes[left_grandgrandchild.unwrap() as usize].set_is_red(false);
                            self.nodes[right_grandchild].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                        (false, true) => unsafe {
                            self.rotate_left(tree_id, right_grandchild as u32);
                            self.rotate_left(tree_id, left_id as u32);
                            self.rotate_right(tree_id, id as u32);
                            // left_grandgrandchild is red, so it always exists
                            self.nodes[right_grandgrandchild.unwrap() as usize].set_is_red(false);
                            self.nodes[right_grandchild].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                    }
                } else {
                    let left_grandchild = self.nodes[left_id].left();
                    let right_grandchild = self.nodes[left_id].right();

                    match (self.is_red(left_grandchild), self.is_red(right_grandchild)) {
                        (false, false) => unsafe {
                            self.nodes[left_id].set_is_red(true);
                            if let Some(parent_id) = self.nodes[id].parent() {
                                self.balance_subtree(tree_id, parent_id as usize);
                            }
                        },
                        (_, true) => unsafe {
                            self.rotate_left(tree_id, left_id as u32);
                            self.rotate_right(tree_id, id as u32);
                            self.nodes[left_id].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                        (true, false) => unsafe {
                            self.nodes[left_grandchild.unwrap() as usize].set_is_red(false);
                            self.rotate_right(tree_id, id as u32);
                            self.nodes[id].set_is_red(false);
                        },
                    }
                }
            }
            Ordering::Less => {
                // right_depth is greater than left_depth, so it >= 1 and therefore right_child
                // always exists
                let right_id = right_child.unwrap() as usize;
                if self.nodes[id].is_red() {
                    debug_assert!(!self.nodes[right_id].is_red());
                    let right_grandchild = self.nodes[right_id].right();
                    let left_grandchild = self.nodes[right_id].left();
                    match (self.is_red(right_grandchild), self.is_red(left_grandchild)) {
                        (false, false) => unsafe {
                            self.nodes[id].set_is_red(false);
                            self.nodes[right_id].set_is_red(true);
                        },
                        (true, _) => unsafe {
                            self.rotate_left(tree_id, id as u32);

                            self.nodes[id].set_is_red(false);
                            self.nodes[right_id].set_is_red(true);
                            // right_grandchild is red, so it always exists
                            self.nodes[right_grandchild.unwrap() as usize].set_is_red(false);
                        },
                        (false, true) => unsafe {
                            self.rotate_right(tree_id, right_id as u32);
                            self.rotate_left(tree_id, id as u32);
                            // right_grandchild is red, so it always exists
                            self.nodes[left_grandchild.unwrap() as usize].set_is_red(false);
                        },
                    }
                } else if self.nodes[right_id].is_red() {
                    debug_assert!(!self.is_red(self.nodes[right_id].right()));
                    debug_assert!(!self.is_red(self.nodes[right_id].left()));
                    // right_depth is greater than left_depth, so it is >= 1
                    // right_child is red and does not affect black height
                    // therefore left and right grandchildren exists
                    let left_grandchild = self.nodes[right_id].left().unwrap() as usize;
                    let right_grandgrandchild = self.nodes[left_grandchild].right();
                    let left_grandgrandchild = self.nodes[left_grandchild].left();

                    match (
                        self.is_red(right_grandgrandchild),
                        self.is_red(left_grandgrandchild),
                    ) {
                        (false, false) => unsafe {
                            self.rotate_left(tree_id, id as u32);
                            self.nodes[id].set_is_red(false);
                            self.nodes[left_grandchild].set_is_red(true);
                        },
                        (true, _) => unsafe {
                            self.rotate_right(tree_id, right_id as u32);
                            self.rotate_left(tree_id, id as u32);
                            // right_grandgrandchild is red, so it always exists
                            self.nodes[right_grandgrandchild.unwrap() as usize].set_is_red(false);
                            self.nodes[left_grandchild].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                        (false, true) => unsafe {
                            self.rotate_right(tree_id, left_grandchild as u32);
                            self.rotate_right(tree_id, right_id as u32);
                            self.rotate_left(tree_id, id as u32);
                            // left_grandgrandchild is red, so it always exists
                            self.nodes[left_grandgrandchild.unwrap() as usize].set_is_red(false);
                            self.nodes[left_grandchild].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                    }
                } else {
                    let right_grandchild = self.nodes[right_id].right();
                    let left_grandchild = self.nodes[right_id].left();

                    match (self.is_red(right_grandchild), self.is_red(left_grandchild)) {
                        (false, false) => unsafe {
                            self.nodes[right_id].set_is_red(true);
                            if let Some(parent_id) = self.nodes[id].parent() {
                                self.balance_subtree(tree_id, parent_id as usize);
                            }
                        },
                        (_, true) => unsafe {
                            self.rotate_right(tree_id, right_id as u32);
                            self.rotate_left(tree_id, id as u32);
                            self.nodes[right_id].set_is_red(false);
                            self.nodes[id].set_is_red(false);
                        },
                        (true, false) => unsafe {
                            // right_grandchild is red, so it always exists
                            self.nodes[right_grandchild.unwrap() as usize].set_is_red(false);
                            self.rotate_left(tree_id, id as u32);
                            self.nodes[id].set_is_red(false);
                        },
                    }
                }
            }
            Ordering::Equal => {
                unreachable!("balance_subtree() should only be called on non ballanced trees. It could be a sign, that the tree was not previously balanced.");
            }
        }
    }

    #[must_use]
    fn black_depth(&self, mut maybe_id: Option<u32>) -> usize {
        let mut depth = 0;
        while let Some(id) = maybe_id {
            if !self.nodes[id as usize].is_red() {
                depth += 1;
            }
            maybe_id = self.nodes[id as usize].left();
        }
        depth
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

        unsafe {
            self.nodes[index].set_parent(allocator_head);
            self.header.set_head(node_index);
        }
    }

    /// Allocates a node
    ///
    /// # Safety
    ///
    /// This function does nothing but allocation. The returned node (if present) is
    /// completely unlinked from the tree and is in the unknown state. The caller must fill the
    /// node with correct data.
    #[must_use]
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

    fn min(&self, mut min_id: usize) -> usize {
        while let Some(id) = self.nodes[min_id].left() {
            min_id = id as usize;
        }
        min_id
    }

    fn max(&self, mut max_id: usize) -> usize {
        while let Some(id) = self.nodes[max_id].right() {
            max_id = id as usize;
        }
        max_id
    }
}

pub struct PairsIterator<
    'a,
    'b,
    K,
    V,
    const KSIZE: usize,
    const VSIZE: usize,
    const MAX_ROOTS: usize,
> where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    next_node: Option<usize>,
    tree: &'a RBForest<'b, K, V, KSIZE, VSIZE, MAX_ROOTS>,
}

impl<'a, 'b, K, V, const KSIZE: usize, const VSIZE: usize, const MAX_ROOTS: usize> Iterator
    for PairsIterator<'a, 'b, K, V, KSIZE, VSIZE, MAX_ROOTS>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    type Item = (K, V);

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node.map(|mut id| {
            let nodes = &self.tree.nodes;

            let key = K::deserialize(&mut dbg!(nodes[id].key.as_slice())).expect("Key corrupted");
            let value = V::deserialize(&mut nodes[id].value.as_slice()).expect("Value corrupted");

            // find next
            if let Some(right_id) = nodes[id].right() {
                self.next_node = Some(self.tree.min(right_id as usize));
            } else {
                self.next_node = None;
                while let Some(parent_id) = nodes[id].parent() {
                    let parent_id = parent_id as usize;
                    if Some(id as u32) == nodes[parent_id].left() {
                        self.next_node = Some(parent_id);
                        break;
                    } else {
                        id = parent_id;
                    }
                }
            }

            (key, value)
        })
    }
}

pub struct KeysIterator<
    'a,
    'b,
    K,
    V,
    const KSIZE: usize,
    const VSIZE: usize,
    const MAX_ROOTS: usize,
> where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    next_node: Option<usize>,
    tree: &'a RBForest<'b, K, V, KSIZE, VSIZE, MAX_ROOTS>,
}

impl<'a, 'b, K, V, const KSIZE: usize, const VSIZE: usize, const MAX_ROOTS: usize> Iterator
    for KeysIterator<'a, 'b, K, V, KSIZE, VSIZE, MAX_ROOTS>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    type Item = K;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node.map(|mut id| {
            let nodes = &self.tree.nodes;

            let key = K::deserialize(&mut dbg!(nodes[id].key.as_slice())).expect("Key corrupted");

            // find next
            if let Some(right_id) = nodes[id].right() {
                self.next_node = Some(self.tree.min(right_id as usize));
            } else {
                self.next_node = None;
                while let Some(parent_id) = nodes[id].parent() {
                    let parent_id = parent_id as usize;
                    if Some(id as u32) == nodes[parent_id].left() {
                        self.next_node = Some(parent_id);
                        break;
                    } else {
                        id = parent_id;
                    }
                }
            }

            key
        })
    }
}

pub struct ValuesIterator<
    'a,
    'b,
    K,
    V,
    const KSIZE: usize,
    const VSIZE: usize,
    const MAX_ROOTS: usize,
> where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    next_node: Option<usize>,
    tree: &'a RBForest<'b, K, V, KSIZE, VSIZE, MAX_ROOTS>,
}

impl<'a, 'b, K, V, const KSIZE: usize, const VSIZE: usize, const MAX_ROOTS: usize> Iterator
    for ValuesIterator<'a, 'b, K, V, KSIZE, VSIZE, MAX_ROOTS>
where
    K: Ord + BorshDeserialize + BorshSerialize,
    V: BorshDeserialize + BorshSerialize,
    [(); mem::size_of::<Header<MAX_ROOTS>>()]: Sized,
{
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_node.map(|mut id| {
            let nodes = &self.tree.nodes;

            let value = V::deserialize(&mut nodes[id].value.as_slice()).expect("Value corrupted");

            // find next
            if let Some(right_id) = nodes[id].right() {
                self.next_node = Some(self.tree.min(right_id as usize));
            } else {
                self.next_node = None;
                while let Some(parent_id) = nodes[id].parent() {
                    let parent_id = parent_id as usize;
                    if Some(id as u32) == nodes[parent_id].left() {
                        self.next_node = Some(parent_id);
                        break;
                    } else {
                        id = parent_id;
                    }
                }
            }

            value
        })
    }
}

#[cfg(test)]
pub(super) mod tests;
