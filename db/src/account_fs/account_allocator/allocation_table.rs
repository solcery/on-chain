use bytemuck::{Pod, Zeroable};
use std::fmt;

// Header magic, used to check the validity of account data strustures.
// "Solcery_DB_Account_Header" in ASCII bytes.
const ACCOUNT_HEADER_MAGIC: [u8; 25] = [
    83, 111, 108, 99, 101, 114, 121, 95, 68, 66, 95, 65, 99, 99, 111, 117, 110, 116, 95, 72, 101,
    97, 100, 101, 114,
];

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct AllocationTable {
    magic: [u8; 25],
    inodes_count: [u8; 2],
    inodes_max: [u8; 2],
    id_autoincrement: [u8; 4],
}

impl AllocationTable {
    pub fn check_magic(&self) -> bool {
        self.magic == ACCOUNT_HEADER_MAGIC
    }

    pub fn inodes_count(&self) -> usize {
        u16::from_be_bytes(self.inodes_count) as usize
    }

    pub fn inodes_max(&self) -> usize {
        u16::from_be_bytes(self.inodes_max) as usize
    }

    pub fn generate_id(&mut self) -> u32 {
        let id = u32::from_be_bytes(self.id_autoincrement);
        self.id_autoincrement = u32::to_be_bytes(id + 1);
        id
    }

    pub unsafe fn set_inodes_count(&mut self, inodes_count: usize) {
        assert!(inodes_count < u16::MAX as usize);
        let inodes_count = inodes_count as u16;
        self.inodes_count = u16::to_be_bytes(inodes_count);
    }

    pub unsafe fn fill(&mut self, inodes_max: usize) {
        assert!(inodes_max < u16::MAX as usize);

        self.magic = ACCOUNT_HEADER_MAGIC;

        let inodes_max = inodes_max as u16;
        self.inodes_max = u16::to_be_bytes(inodes_max);

        self.inodes_count = u16::to_be_bytes(1);
        self.id_autoincrement = u32::to_be_bytes(0);
    }
}

impl fmt::Debug for AllocationTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AllocationTable")
            .field("inode_count", &self.inodes_count())
            .field("inode_max", &self.inodes_max())
            .field(
                "id_autoincrement",
                &u32::from_be_bytes(self.id_autoincrement),
            )
            .finish()
    }
}
