use std::cmp::Ord;
use std::cmp::Ordering;
use std::marker::PhantomData;
//use bitvec::array::BitArray;
use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
struct Node<const KSIZE: usize, const VSIZE: usize> {
    key: [u8; KSIZE],
    value: [u8; VSIZE],
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
    fn key(&self) -> &[u8; KSIZE] {
        &self.key
    }

    fn value(&self) -> &[u8; VSIZE] {
        &self.value
    }

    // TODO: reimplement this functions with macros to avoid code duplication
    fn left_idx(&self) -> Option<u32> {
        // bit position of the flag is 0
        if self.flags & 0b0001 == 1 {
            Some(u32::from_be_bytes(self.left))
        } else {
            None
        }
    }

    fn rignt_idx(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b0010 == 1 {
            Some(u32::from_be_bytes(self.right))
        } else {
            None
        }
    }

    fn parent_idx(&self) -> Option<u32> {
        // bit position of the flag is 2
        if self.flags & 0b0100 == 1 {
            Some(u32::from_be_bytes(self.parent))
        } else {
            None
        }
    }
    fn is_red(&self) -> bool {
        // bit position of the flag is 3
        self.flags & 0b1000 == 1
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
                // All flags but 1th will remain the same, which will be set to 0.
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
                // All flags but 2th will remain the same, which will be set to 0.
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
}

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
struct Header {
    k_size: [u8; 4],
    v_size: [u8; 4],
    max_nodes: [u8; 4],
    root: [u8; 4],
    is_root_present: u8,
    empty_node_ptr: [u8; 4],
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;

    #[test]
    fn alignment_node() {
        assert_eq!(mem::align_of::<Node<8, 8>>(), 1);
    }
}
