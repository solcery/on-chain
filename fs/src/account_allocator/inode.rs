use bytemuck::{Pod, Zeroable};
use std::fmt;

/// Data stucture containing all the metadata of a single data chunk
#[repr(C)]
#[derive(Default, Pod, Clone, Copy, Zeroable, PartialEq, Eq)]
pub struct Inode {
    /// Flag layout:
    /// 0. is node occupied
    /// 1-7. not used
    flags: u8,
    /// index of the first byte of the data chunk, encoded as big-endian `u32`
    start_idx: [u8; 4],
    /// index of the last + 1 byte of the data chunk, encoded as big-endian `u32`
    end_idx: [u8; 4],
    /// id of the chunk, encoded as big-endian `u32`
    id: [u8; 4],
}

impl Inode {
    pub fn start_idx(&self) -> usize {
        u32::from_be_bytes(self.start_idx) as usize
    }

    pub fn end_idx(&self) -> usize {
        u32::from_be_bytes(self.end_idx) as usize
    }

    pub fn id(&self) -> Option<u32> {
        if self.flags == 0 {
            Some(u32::from_be_bytes(self.id))
        } else {
            None
        }
    }

    pub fn is_occupied(&self) -> bool {
        self.flags == 0
    }

    pub fn len(&self) -> usize {
        let start = u32::from_be_bytes(self.start_idx) as usize;
        let end = u32::from_be_bytes(self.end_idx) as usize;
        end - start
    }

    pub fn unoccupy(&mut self) {
        self.flags = 1;
    }

    pub unsafe fn occupy(&mut self, id: u32) {
        self.flags = 0;
        self.id = u32::to_be_bytes(id);
    }

    /// Generate new [`Inode`] with initial values proper values
    pub unsafe fn from_raw_parts(start_idx: usize, end_idx: usize, maybe_id: Option<u32>) -> Self {
        let start_idx = u32::to_be_bytes(start_idx as u32);
        let end_idx = u32::to_be_bytes(end_idx as u32);
        let id;
        let flags;
        match maybe_id {
            Some(num) => {
                id = u32::to_be_bytes(num);
                flags = 0;
            }
            None => {
                id = u32::to_be_bytes(0);
                flags = 1;
            }
        }

        Self {
            start_idx,
            end_idx,
            id,
            flags,
        }
    }

    /// initialize the given [`Inode`] with proper values
    pub unsafe fn fill(&mut self, start_idx: usize, end_idx: usize, id: u32, flags: bool) {
        self.start_idx = u32::to_be_bytes(start_idx as u32);
        self.end_idx = u32::to_be_bytes(end_idx as u32);
        self.id = u32::to_be_bytes(id as u32);
        if flags {
            self.flags = 0;
        } else {
            self.flags = 1;
        }
    }
}

impl fmt::Debug for Inode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Inode")
            .field("start_idx", &self.start_idx())
            .field("end_idx", &self.end_idx())
            .field("id", &self.id())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::mem;

    #[test]
    fn table_size() {
        assert_eq!(mem::size_of::<Inode>(), 13);
    }
}
