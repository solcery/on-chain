use bytemuck::{Pod, Zeroable};
use std::fmt;

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
pub struct Node<const KSIZE: usize, const VSIZE: usize> {
    pub key: [u8; KSIZE],
    pub value: [u8; VSIZE],
    size: [u8; 4],
    left: [u8; 4],
    right: [u8; 4],
    /// Flag layout:
    ///
    /// 0. is_left_present
    /// 1. is_right_present
    /// 2. is_red
    flags: u8, // Will change it to BitArray, than it become Pod
               // Total size KSIZE + VSIZE + 17
}

unsafe impl<const KSIZE: usize, const VSIZE: usize> Pod for Node<KSIZE, VSIZE> {}

impl<const KSIZE: usize, const VSIZE: usize> Node<KSIZE, VSIZE> {
    pub fn size(&self) -> u32 {
        u32::from_be_bytes(self.size)
    }

    // TODO: reimplement this functions with macros to avoid code duplication
    pub fn left(&self) -> Option<u32> {
        // bit position of the flag is 0
        if self.flags & 0b001 == 1 {
            Some(u32::from_be_bytes(self.left))
        } else {
            None
        }
    }

    pub fn right(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b010 != 0 {
            Some(u32::from_be_bytes(self.right))
        } else {
            None
        }
    }

    pub fn is_red(&self) -> bool {
        // bit position of the flag is 2
        self.flags & 0b100 != 0
    }

    pub unsafe fn set_size(&mut self, size: u32) {
        self.size = u32::to_be_bytes(size);
    }

    pub unsafe fn set_left(&mut self, left: Option<u32>) {
        // bit position of the flag is 0
        match left {
            Some(idx) => {
                self.left = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b001;
            }
            None => {
                // All flags but 0th will remain the same, which will be set to 0.
                self.flags = self.flags & 0b110;
            }
        }
    }

    pub unsafe fn set_right(&mut self, right: Option<u32>) {
        // bit position of the flag is 1
        match right {
            Some(idx) => {
                self.right = u32::to_be_bytes(idx);
                self.flags = self.flags | 0b010;
            }
            None => {
                // All flags but 1st will remain the same, which will be set to 0.
                self.flags = self.flags & 0b101;
            }
        }
    }

    pub unsafe fn set_is_red(&mut self, is_red: bool) {
        if is_red {
            self.flags = self.flags | 0b100;
        } else {
            self.flags = self.flags & 0b011;
        }
    }

    pub unsafe fn init_node(&mut self) {
        self.size = u32::to_be_bytes(1);
        // Flags set:
        // left = None
        // right = None
        // is_red = true
        self.flags = 0b100;
        self.key.fill(0);
        self.value.fill(0);
    }

    #[cfg(test)]
    unsafe fn from_raw_parts(
        key: [u8; KSIZE],
        value: [u8; VSIZE],
        size: u32,
        left: Option<u32>,
        right: Option<u32>,
        is_red: bool,
    ) -> Self {
        let size = u32::to_be_bytes(size);
        let mut flags = 0b000;

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

        if is_red {
            flags = flags | 0b100;
        }

        Self {
            key,
            value,
            size,
            left,
            right,
            flags,
        }
    }
}

impl<const KSIZE: usize, const VSIZE: usize> fmt::Debug for Node<KSIZE, VSIZE> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Node")
            .field("key", &self.key)
            .field("value", &self.value)
            .field("size", &self.size())
            .field("left", &self.left())
            .field("right", &self.right())
            .field("is_red", &self.is_red())
            .finish()
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
                    unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, false) };

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
                assert_eq!(node.is_red(), false);
            }
        };
    }

    option_test!(left);
    option_test!(right);

    #[test]
    fn init_node() {
        let mut node = unsafe { Node::<1, 1>::from_raw_parts([1], [2], 3, None, None, false) };

        unsafe {
            node.init_node();
        }

        assert_eq!(node.size(), 1);
        assert_eq!(node.left(), None);
        assert_eq!(node.right(), None);
        assert_eq!(node.is_red(), true);
    }
}
