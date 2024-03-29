use bytemuck::{Pod, Zeroable};
use std::fmt;

// Header magic, used to check the validity of account data structures
const ACCOUNT_HEADER_MAGIC: [u8; 25] = *b"Solcery_FS_Account_Header";

/// Metadata required to operate with account data
#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct AllocationTable {
    /// Magic value, should be equal to [`ACCOUNT_HEADER_MAGIC`]
    magic: [u8; 25],
    /// Number of used inodes in the inode table, encoded as big-endian `u16`
    inodes_count: [u8; 2],
    /// Maximum number of inodes in the table, encoded as big-endian `u16`
    inodes_max: [u8; 2],
    /// Internal monotonic counter used to create unique chunk indexes, encoded as big-endian `u32`
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

    pub(super) fn id_autoincrement(&self) -> u32 {
        u32::from_be_bytes(self.id_autoincrement)
    }

    pub fn set_inodes_count(&mut self, inodes_count: usize) {
        assert!(inodes_count < u16::MAX as usize);
        let inodes_count = inodes_count as u16;
        self.inodes_count = u16::to_be_bytes(inodes_count);
    }

    /// initialize the given [`AllocationTable`] with proper values
    pub fn fill(&mut self, inodes_max: usize) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::mem;

    #[test]
    fn table_size() {
        assert_eq!(mem::size_of::<AllocationTable>(), 33);
    }
}
