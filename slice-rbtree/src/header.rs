use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Header {
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
    pub fn head(&self) -> Option<u32> {
        // bit position of the flag is 1
        if self.flags & 0b0010 != 0 {
            Some(u32::from_be_bytes(self.head))
        } else {
            None
        }
    }

    pub unsafe fn set_root(&mut self, root: Option<u32>) {
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

    pub unsafe fn set_head(&mut self, head: Option<u32>) {
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
    pub unsafe fn fill(
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
