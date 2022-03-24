use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
//use bitvec::array::BitArray;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

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
    use pretty_assertions::assert_eq;

    #[test]
    fn left() {
        let mut node =
            unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, None, false) };
        assert_eq!(node.left(), None);

        unsafe {
            node.set_left(Some(1));
        }
        assert_eq!(node.left(), Some(1));

        unsafe {
            node.set_left(Some(2));
        }
        assert_eq!(node.left(), Some(2));

        // check other values

        assert_eq!(node.key, [1]);
        assert_eq!(node.value, [2]);
        assert_eq!(node.size(), 3);
        assert_eq!(node.right(), None);
        assert_eq!(node.parent(), None);
        assert_eq!(node.is_red(), false);
    }

    #[test]
    fn right() {
        let mut node =
            unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, None, false) };
        assert_eq!(node.right(), None);

        unsafe {
            node.set_right(Some(1));
        }
        dbg!(&node.right);
        dbg!(&node.flags);
        assert_eq!(node.right(), Some(1));

        unsafe {
            node.set_right(Some(2));
        }
        assert_eq!(node.right(), Some(2));

        // check other values

        assert_eq!(node.key, [1]);
        assert_eq!(node.value, [2]);
        assert_eq!(node.size(), 3);
        assert_eq!(node.left(), None);
        assert_eq!(node.parent(), None);
        assert_eq!(node.is_red(), false);
    }
    #[test]
    fn parent() {
        let mut node =
            unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, None, false) };
        assert_eq!(node.parent(), None);

        unsafe {
            node.set_parent(Some(1));
        }
        assert_eq!(node.parent(), Some(1));

        unsafe {
            node.set_parent(Some(2));
        }
        assert_eq!(node.parent(), Some(2));

        // check other values

        assert_eq!(node.key, [1]);
        assert_eq!(node.value, [2]);
        assert_eq!(node.size(), 3);
        assert_eq!(node.right(), None);
        assert_eq!(node.right(), None);
        assert_eq!(node.is_red(), false);
    }
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

    fn root_index(&self) -> Option<u32> {
        // bit position of the flag is 0
        if self.flags & 0b0001 == 1 {
            Some(u32::from_be_bytes(self.root))
        } else {
            None
        }
    }

    fn head_index(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b0010 == 1 {
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
