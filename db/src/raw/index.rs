use borsh::BorshDeserialize;
use borsh::BorshSerialize;
use bytemuck::{Pod, Zeroable};
use std::fmt;
use std::mem;

use super::column::ColumnHeader;
use solcery_data_types::db::schema::DataType;

const INDEX_MAGIC: [u8; 16] = *b"Solcery_DB_Index";
const CURRENT_VERSION: [u8; 2] = u16::to_be_bytes(0);

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct Index {
    magic: [u8; 16],
    db_version: [u8; 2],
    table_name: [u8; 64],
    primary_key_type: u8,
    column_count: u8,
    column_max: u8,
    column_id_autoincrement: [u8; 4],
    max_rows: [u8; 4],
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

    pub fn primary_key_type(&self) -> DataType {
        match DataType::try_from(self.primary_key_type) {
            Ok(val) => val,
            Err(_) => unreachable!("Unknown primary key type, it is a sign of data corruption"),
        }
    }

    pub fn table_name(&self) -> String {
        String::deserialize(&mut self.table_name.as_slice()).unwrap()
    }

    pub fn version(&self) -> u16 {
        u16::from_be_bytes(self.db_version)
    }

    pub fn generate_id(&mut self) -> u32 {
        let id = u32::from_be_bytes(self.column_id_autoincrement);
        self.column_id_autoincrement = u32::to_be_bytes(id + 1);
        id
    }

    pub const fn size(num_columns: usize) -> usize {
        mem::size_of::<Self>() + mem::size_of::<ColumnHeader>() * num_columns
    }

    pub const fn columns_size(&self) -> usize {
        mem::size_of::<ColumnHeader>() * self.column_max as usize
    }

    pub fn max_rows(&self) -> usize {
        u32::from_be_bytes(self.max_rows) as usize
    }

    pub unsafe fn set_column_count(&mut self, count: usize) {
        assert!(count <= u8::MAX as usize);
        self.column_count = count as u8;
    }

    pub unsafe fn fill(
        &mut self,
        table_name: &str,
        primary_key_type: DataType,
        column_max: usize,
        max_rows: usize,
    ) {
        self.magic = INDEX_MAGIC;
        self.db_version = CURRENT_VERSION;
        table_name
            .serialize(&mut self.table_name.as_mut_slice())
            .unwrap(); // TODO: Document unwrap
        self.primary_key_type = u8::from(primary_key_type);
        self.column_count = 0;
        assert!(column_max <= u8::MAX as usize);
        self.column_max = column_max as u8;
        self.column_id_autoincrement = u32::to_be_bytes(0);
        assert!(max_rows <= u32::MAX as usize);
        self.max_rows = u32::to_be_bytes(max_rows as u32);
    }
}

impl fmt::Debug for Index {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Index")
            .field("version", &self.version())
            .field("table_name", &self.table_name())
            .field("primary_key_type", &self.primary_key_type())
            .field("column_count", &self.column_count())
            .field("column_max", &self.column_max())
            .finish()
    }
}
