use borsh::de::BorshDeserialize;
use bytemuck::{Pod, Zeroable};

const INDEX_MAGIC: [u8; 16] = *b"Solcery_DB_Index";

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Index {
    magic: [u8; 16],
    table_name: [u8; 64],
    primary_key_type: u8,
    primary_key_length: u8,
    column_count: u8,
    column_max: u8,
}

impl Index {
    pub fn check_magic(&self) -> bool {
        self.magic == INDEX_MAGIC
    }

    pub fn column_count(&self) -> usize {
        self.column_count as usize
    }

    pub fn column_max(&self) -> usize {
        self.column_max as usize
    }

    pub fn table_name(&self) -> String {
        String::deserialize(&mut self.table_name.as_slice()).unwrap()
    }

    pub unsafe fn set_column_count(&mut self, count: usize) {
        assert!(count <= u8::MAX as usize);
        self.column_count = count as u8;
    }
}
