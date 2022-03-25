use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{cast_mut, cast_slice_mut, Pod, Zeroable};
use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
use std::mem;

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
struct Node<const KSIZE: usize, const VSIZE: usize> {
    pub key: [u8; KSIZE],
    pub value: [u8; VSIZE],
    size: [u8; 4],
    left: [u8; 4],
    right: [u8; 4],
    parent: [u8; 4],
    /// Flag layout:
    ///
    /// 0. is_left_present
    /// 1. is_right_present
    /// 2. is_parent_present
    /// 3. is_red
    flags: u8, // Will change it to BitArray, than it become Pod
               // Total size KSIZE + VSIZE + 17
}

unsafe impl<const KSIZE: usize, const VSIZE: usize> Pod for Node<KSIZE, VSIZE> {}

impl<const KSIZE: usize, const VSIZE: usize> Node<KSIZE, VSIZE> {
    fn size(&self) -> u32 {
        u32::from_be_bytes(self.size)
    }

    // TODO: reimplement this functions with macros to avoid code duplication
    fn left(&self) -> Option<u32> {
        // bit position of the flag is 0
        if self.flags & 0b0001 == 1 {
            Some(u32::from_be_bytes(self.left))
        } else {
            None
        }
    }

    fn right(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b0010 != 0 {
            Some(u32::from_be_bytes(self.right))
        } else {
            None
        }
    }

    fn parent(&self) -> Option<u32> {
        // bit position of the flag is 2
        if self.flags & 0b0100 != 0 {
            Some(u32::from_be_bytes(self.parent))
        } else {
            None
        }
    }
    fn is_red(&self) -> bool {
        // bit position of the flag is 3
        self.flags & 0b1000 != 0
    }

    unsafe fn set_size(&mut self, size: u32) {
        self.left = u32::to_be_bytes(size);
    }

    unsafe fn set_left(&mut self, left: Option<u32>) {
        // bit position of the flag is 0
        match left {
            Some(idx) => {
                self.left = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b0001;
            }
            None => {
                // All flags but 0th will remain the same, which will be set to 0.
                self.flags = self.flags & 0b1110;
            }
        }
    }

    unsafe fn set_right(&mut self, right: Option<u32>) {
        // bit position of the flag is 1
        match right {
            Some(idx) => {
                self.right = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b0010;
            }
            None => {
                // All flags but 1st will remain the same, which will be set to 0.
                self.flags = self.flags & 0b1101;
            }
        }
    }

    unsafe fn set_parent(&mut self, parent: Option<u32>) {
        // bit position of the flag is 2
        match parent {
            Some(idx) => {
                self.parent = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b0100;
            }
            None => {
                // All flags but 2nd will remain the same, which will be set to 0.
                self.flags = self.flags & 0b1011;
            }
        }
    }

    unsafe fn set_is_color(&mut self, is_red: bool) {
        if is_red {
            self.flags = self.flags | 0b1000;
        } else {
            self.flags = self.flags & 0b0111;
        }
    }

    unsafe fn from_raw_parts(
        key: [u8; KSIZE],
        value: [u8; VSIZE],
        size: u32,
        left: Option<u32>,
        right: Option<u32>,
        parent: Option<u32>,
        is_red: bool,
    ) -> Self {
        let size = u32::to_be_bytes(size);
        let mut flags = 0b00;

        let left = match left {
            Some(index) => {
                flags = flags | 0b0001;
                u32::to_be_bytes(index)
            }
            None => u32::to_be_bytes(0),
        };

        let right = match right {
            Some(index) => {
                flags = flags | 0b0010;
                u32::to_be_bytes(index)
            }
            None => u32::to_be_bytes(0),
        };

        let parent = match parent {
            Some(index) => {
                flags = flags | 0b0100;
                u32::to_be_bytes(index)
            }
            None => u32::to_be_bytes(0),
        };

        if is_red {
            flags = flags | 0b1000;
        }

        Self {
            key,
            value,
            size,
            left,
            right,
            parent,
            flags,
        }
    }
}

#[cfg(test)]
mod node_tests {
    use super::*;
    use paste::paste;
    use pretty_assertions::assert_eq;

    macro_rules! option_test {
        ($method:ident) => {
            #[test]
            fn $method() {
                let mut node =
                    unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, None, false) };

                unsafe {
                    paste! {
                        node.[<set_ $method>](Some(1));
                    }
                }
                assert_eq!(node.$method(), Some(1));

                unsafe {
                    paste! {
                        node.[<set_ $method>](Some(2));
                    }
                }
                assert_eq!(node.$method(), Some(2));
                unsafe {
                    paste! {
                        node.[<set_ $method>](None);
                    }
                }

                assert_eq!(node.key, [1]);
                assert_eq!(node.value, [2]);
                assert_eq!(node.size(), 3);
                assert_eq!(node.left(), None);
                assert_eq!(node.right(), None);
                assert_eq!(node.parent(), None);
                assert_eq!(node.is_red(), false);
            }
        };
    }

    option_test!(left);
    option_test!(right);
    option_test!(parent);
}

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
struct Header {
    k_size: [u8; 4],
    v_size: [u8; 4],
    max_nodes: [u8; 4],
    root: [u8; 4],
    /// Flag layout:
    ///
    /// 0. is_root_present
    /// 1. is_head_present
    flags: u8,
    /// head of the linked list of empty nodes
    head: [u8; 4],
}

impl Header {
    fn k_size(&self) -> u32 {
        u32::from_be_bytes(self.k_size)
    }

    fn v_size(&self) -> u32 {
        u32::from_be_bytes(self.v_size)
    }

    fn max_nodes(&self) -> u32 {
        u32::from_be_bytes(self.max_nodes)
    }

    fn root(&self) -> Option<u32> {
        // bit position of the flag is 0
        if self.flags & 0b0001 != 0 {
            Some(u32::from_be_bytes(self.root))
        } else {
            None
        }
    }

    fn head(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b0010 != 0 {
            Some(u32::from_be_bytes(self.head))
        } else {
            None
        }
    }

    unsafe fn set_root(&mut self, root: Option<u32>) {
        // bit position of the flag is 0
        match root {
            Some(idx) => {
                self.root = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b0001;
            }
            None => {
                // All flags but 0th will remain the same, which will be set to 0.
                self.flags = self.flags & 0b1110;
            }
        }
    }

    unsafe fn set_head(&mut self, head: Option<u32>) {
        // bit position of the flag is 1
        match head {
            Some(idx) => {
                self.head = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b0010;
            }
            None => {
                // All flags but 1st will remain the same, which will be set to 0.
                self.flags = self.flags & 0b1101;
            }
        }
    }

    unsafe fn from_raw_parts(
        k_size: u32,
        v_size: u32,
        max_nodes: u32,
        root: Option<u32>,
        head: Option<u32>,
    ) -> Self {
        let k_size = u32::to_be_bytes(k_size);
        let v_size = u32::to_be_bytes(v_size);
        let max_nodes = u32::to_be_bytes(max_nodes);
        let mut flags = 0b00;

        let root = match root {
            Some(index) => {
                flags = flags | 0b01;
                u32::to_be_bytes(index)
            }
            None => u32::to_be_bytes(0),
        };

        let head = match head {
            Some(index) => {
                flags = flags | 0b10;
                u32::to_be_bytes(index)
            }
            None => u32::to_be_bytes(0),
        };
        Self {
            k_size,
            v_size,
            max_nodes,
            root,
            head,
            flags,
        }
    }

    /// This function guarantees, that the header will be initialized in fully known state
    unsafe fn fill(
        &mut self,
        k_size: u32,
        v_size: u32,
        max_nodes: u32,
        root: Option<u32>,
        head: Option<u32>,
    ) {
        self.k_size = u32::to_be_bytes(k_size);
        self.v_size = u32::to_be_bytes(v_size);
        self.max_nodes = u32::to_be_bytes(max_nodes);
        self.flags = 0b00;
        self.set_head(head);
        self.set_root(root);
    }
}

#[cfg(test)]
mod header_tests {
    use super::*;
    use paste::paste;
    use pretty_assertions::assert_eq;

    macro_rules! option_test {
        ($method:ident) => {
            #[test]
            fn $method() {
                let mut head = unsafe { Header::from_raw_parts(1, 2, 3, None, None) };

                unsafe {
                    paste! {
                        head.[<set_ $method>](Some(1));
                    }
                }
                assert_eq!(head.$method(), Some(1));

                unsafe {
                    paste! {
                        head.[<set_ $method>](Some(2));
                    }
                }
                assert_eq!(head.$method(), Some(2));
                unsafe {
                    paste! {
                        head.[<set_ $method>](None);
                    }
                }

                assert_eq!(head.k_size(), 1);
                assert_eq!(head.v_size(), 2);
                assert_eq!(head.max_nodes(), 3);
                assert_eq!(head.root(), None);
                assert_eq!(head.head(), None);
            }
        };
    }

    option_test!(root);
    option_test!(head);
}

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
    unsafe fn allocate_node(&mut self) -> Option<usize> {
        let allocator_head = self.header.head();
        match allocator_head {
            Some(index) => {
                let new_head = self.nodes[index as usize].parent();
                self.header.set_head(new_head);
                Some(index as usize)
            }
            None => None,
        }
    }
}

pub enum Error {
    TooSmall,
    WrongNodePoolSize,
}
