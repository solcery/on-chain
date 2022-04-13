use bytemuck::{Pod, Zeroable};
use std::fmt;

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Header {
    k_size: [u8; 2],
    v_size: [u8; 2],
    max_nodes: [u8; 4],
    /// root of the tree
    root: [u8; 4],
    /// head of the linked list of empty nodes
    head: [u8; 4],
}

impl Header {
    pub fn k_size(&self) -> u16 {
        u16::from_be_bytes(self.k_size)
    }

    pub fn v_size(&self) -> u16 {
        u16::from_be_bytes(self.v_size)
    }

    pub fn max_nodes(&self) -> u32 {
        u32::from_be_bytes(self.max_nodes)
    }

    pub fn root(&self) -> Option<u32> {
        let num = u32::from_be_bytes(self.root);
        if num == u32::MAX {
            None
        } else {
            Some(num)
        }
    }

    pub fn head(&self) -> Option<u32> {
        let num = u32::from_be_bytes(self.head);
        if num == u32::MAX {
            None
        } else {
            Some(num)
        }
    }

    pub unsafe fn set_root(&mut self, root: Option<u32>) {
        match root {
            Some(idx) => {
                assert!(idx < u32::MAX);
                self.root = u32::to_be_bytes(idx);
            }
            None => {
                self.root = u32::to_be_bytes(u32::MAX);
            }
        }
    }

    pub unsafe fn set_head(&mut self, head: Option<u32>) {
        match head {
            Some(idx) => {
                assert!(idx < u32::MAX);
                self.head = u32::to_be_bytes(idx);
            }
            None => {
                self.head = u32::to_be_bytes(u32::MAX);
            }
        }
    }

    /// This function guarantees, that the header will be initialized in fully known state
    pub unsafe fn fill(
        &mut self,
        k_size: u16,
        v_size: u16,
        max_nodes: u32,
        root: Option<u32>,
        head: Option<u32>,
    ) {
        self.k_size = u16::to_be_bytes(k_size);
        self.v_size = u16::to_be_bytes(v_size);
        self.max_nodes = u32::to_be_bytes(max_nodes);
        unsafe {
            self.set_head(head);
            self.set_root(root);
        }
    }

    #[cfg(test)]
    unsafe fn from_raw_parts(
        k_size: u16,
        v_size: u16,
        max_nodes: u32,
        root: Option<u32>,
        head: Option<u32>,
    ) -> Self {
        let k_size = u16::to_be_bytes(k_size);
        let v_size = u16::to_be_bytes(v_size);
        let max_nodes = u32::to_be_bytes(max_nodes);

        let root = match root {
            Some(index) => u32::to_be_bytes(index),
            None => u32::to_be_bytes(u32::MAX),
        };

        let head = match head {
            Some(index) => u32::to_be_bytes(index),
            None => u32::to_be_bytes(u32::MAX),
        };
        Self {
            k_size,
            v_size,
            max_nodes,
            root,
            head,
        }
    }
}

impl fmt::Debug for Header {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Header")
            .field("k_size", &self.k_size())
            .field("v_size", &self.v_size())
            .field("max_nodes", &self.max_nodes())
            .field("root", &self.root())
            .field("head", &self.head())
            .finish()
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
