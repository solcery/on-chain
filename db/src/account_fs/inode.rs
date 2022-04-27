use bytemuck::{Pod, Zeroable};
use std::fmt;

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable, PartialEq)]
pub struct Inode {
    occupied: u8, // == 0 then inode is occupied
    start_idx: [u8; 4],
    end_idx: [u8; 4],
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
        if self.occupied == 0 {
            Some(u32::from_be_bytes(self.id))
        } else {
            None
        }
    }

    pub fn occupied(&self) -> bool {
        self.occupied == 0
    }

    pub fn len(&self) -> usize {
        let start = u32::from_be_bytes(self.start_idx) as usize;
        let end = u32::from_be_bytes(self.end_idx) as usize;
        end - start
    }

    pub fn unoccupy(&mut self) {
        self.occupied = 1;
    }

    pub unsafe fn occupy(&mut self, id: u32) {
        self.occupied = 1;
        self.id = u32::to_be_bytes(id);
    }

    pub unsafe fn from_raw_parts(start_idx: usize, end_idx: usize, maybe_id: Option<u32>) -> Self {
        let start_idx = u32::to_be_bytes(start_idx as u32);
        let end_idx = u32::to_be_bytes(end_idx as u32);
        let id;
        let occupied;
        match maybe_id {
            Some(num) => {
                id = u32::to_be_bytes(num);
                occupied = 0;
            }
            None => {
                id = u32::to_be_bytes(0);
                occupied = 1;
            }
        }

        Self {
            start_idx,
            end_idx,
            id,
            occupied,
        }
    }

    pub unsafe fn fill(&mut self, start_idx: usize, end_idx: usize, id: u32, occupied: bool) {
        self.start_idx = u32::to_be_bytes(start_idx as u32);
        self.end_idx = u32::to_be_bytes(end_idx as u32);
        self.id = u32::to_be_bytes(id as u32);
        if occupied {
            self.occupied = 0;
        } else {
            self.occupied = 1;
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

impl Default for Inode {
    fn default() -> Self {
        Self {
            occupied: 0,
            start_idx: [0, 0, 0, 0],
            end_idx: [0, 0, 0, 0],
            id: [0, 0, 0, 0],
        }
    }
}
