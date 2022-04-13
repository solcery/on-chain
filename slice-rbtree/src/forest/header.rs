use bytemuck::{Pod, Zeroable};
use std::fmt;

#[repr(C)]
#[derive(Clone, Copy, Zeroable)]
pub struct Header<const MAX_ROOTS: usize> {
    k_size: [u8; 2],
    v_size: [u8; 2],
    max_nodes: [u8; 4],
    /// array of roots of the tree
    roots: [[u8; 4]; MAX_ROOTS],
    /// head of the linked list of empty nodes
    head: [u8; 4],
}

unsafe impl<const MAX_ROOTS: usize> Pod for Header<MAX_ROOTS> {}

impl<const MAX_ROOTS: usize> Header<MAX_ROOTS> {
    pub fn k_size(&self) -> u16 {
        u16::from_be_bytes(self.k_size)
    }

    pub fn v_size(&self) -> u16 {
        u16::from_be_bytes(self.v_size)
    }

    pub fn max_nodes(&self) -> u32 {
        u32::from_be_bytes(self.max_nodes)
    }

    pub fn root(&self, id: usize) -> Option<u32> {
        let num = u32::from_be_bytes(self.roots[id]);
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

    pub unsafe fn set_root(&mut self, id: usize, root: Option<u32>) {
        match root {
            Some(idx) => {
                assert!(idx < u32::MAX);
                self.roots[id] = u32::to_be_bytes(idx);
            }
            None => {
                self.roots[id] = u32::to_be_bytes(u32::MAX);
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
        roots: [Option<u32>; MAX_ROOTS],
        head: Option<u32>,
    ) {
        self.k_size = u16::to_be_bytes(k_size);
        self.v_size = u16::to_be_bytes(v_size);
        self.max_nodes = u32::to_be_bytes(max_nodes);
        unsafe {
            self.set_head(head);
            for (id, root) in roots.into_iter().enumerate() {
                self.set_root(id, root);
            }
        }
    }

    #[cfg(test)]
    unsafe fn from_raw_parts(
        k_size: u16,
        v_size: u16,
        max_nodes: u32,
        roots: &[Option<u32>; MAX_ROOTS],
        head: Option<u32>,
    ) -> Self {
        let k_size = u16::to_be_bytes(k_size);
        let v_size = u16::to_be_bytes(v_size);
        let max_nodes = u32::to_be_bytes(max_nodes);

        let roots = roots.map(|root| match root {
            Some(index) => u32::to_be_bytes(index),
            None => u32::to_be_bytes(u32::MAX),
        });

        let head = match head {
            Some(index) => u32::to_be_bytes(index),
            None => u32::to_be_bytes(u32::MAX),
        };

        Self {
            k_size,
            v_size,
            max_nodes,
            roots,
            head,
        }
    }
}

//impl fmt::Debug for Header {
//fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//f.debug_struct("Header")
//.field("k_size", &self.k_size())
//.field("v_size", &self.v_size())
//.field("max_nodes", &self.max_nodes())
//.field("head", &self.head())
//.finish()
//.debug_list("Roots")
//.entries(&self.roots.iter().map(|x| {
//let num = u32::from_be_bytes(x);
//if num == u32::MAX {
//None
//} else {
//Some(num)
//}
//}))
//.finish()
//}
//}

#[cfg(test)]
mod header_tests {
    use super::*;
    use paste::paste;
    use pretty_assertions::assert_eq;

    #[test]
    fn head() {
        let mut head = unsafe { Header::from_raw_parts(1, 2, 3, &[None], None) };

        unsafe {
            head.set_head(Some(1));
        }
        assert_eq!(head.head(), Some(1));

        unsafe {
            head.set_head(Some(2));
        }
        assert_eq!(head.head(), Some(2));
        unsafe {
            paste! {
                head.set_head(None);
            }
        }

        assert_eq!(head.k_size(), 1);
        assert_eq!(head.v_size(), 2);
        assert_eq!(head.max_nodes(), 3);
        assert_eq!(head.root(0), None);
        assert_eq!(head.head(), None);
    }

    #[test]
    fn roots() {
        let mut head = unsafe { Header::from_raw_parts(1, 2, 3, &[None, None], None) };

        unsafe {
            head.set_root(0, Some(1));
        }
        assert_eq!(head.root(0), Some(1));
        assert_eq!(head.root(1), None);

        unsafe {
            head.set_root(1, Some(2));
        }
        assert_eq!(head.root(1), Some(2));
        assert_eq!(head.root(0), Some(1));
        unsafe {
            head.set_root(0, None);
            head.set_root(1, None);
        }

        assert_eq!(head.k_size(), 1);
        assert_eq!(head.v_size(), 2);
        assert_eq!(head.max_nodes(), 3);
        assert_eq!(head.root(0), None);
        assert_eq!(head.root(1), None);
        assert_eq!(head.head(), None);
    }
}
