use borsh::BorshDeserialize;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use std::fmt;

use account_fs::SegmentId;
use solcery_data_types::db::schema::{ColumnType, DataType};

const NAME_LEN: usize = 64;

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct ColumnHeader {
    name: [u8; NAME_LEN],
    id: [u8; 4],
    value_type: u8,
    account_pubkey: [u8; 32],
    segment_id: [u8; 4],
    column_type: u8, // I'm sure, that we'll never invent more than 256 table types
}

impl ColumnHeader {
    pub fn name(&self) -> String {
        String::deserialize(&mut self.name.as_slice()).unwrap()
    }

    pub fn id(&self) -> u32 {
        u32::from_be_bytes(self.id)
    }

    pub fn value_type(&self) -> DataType {
        DataType::try_from(self.value_type)
            .expect("Unknown data type, it is a sign of data corruption")
    }

    pub fn segment_id(&self) -> SegmentId {
        SegmentId {
            pubkey: Pubkey::new_from_array(self.account_pubkey),
            id: u32::from_be_bytes(self.segment_id),
        }
    }

    pub fn column_type(&self) -> ColumnType {
        ColumnType::try_from(self.value_type)
            .expect("Unknown column type, it is a sign of data corruption")
    }

    pub unsafe fn new(
        name: &str,
        id: u32,
        segment_id: SegmentId,
        value_type: DataType,
        column_type: ColumnType,
    ) -> Self {
        assert!(name.len() <= NAME_LEN);
        let mut name_bytes: [u8; NAME_LEN] = [0; NAME_LEN];
        let (used_bytes, _) = name_bytes.split_at_mut(name.len());
        used_bytes.copy_from_slice(name.as_bytes());
        let value_type = u8::from(value_type);
        let column_type = u8::from(column_type);
        let account_pubkey = segment_id.pubkey.to_bytes();
        let segment_id = segment_id.id.to_be_bytes();
        let id = id.to_be_bytes();

        Self {
            id,
            name: name_bytes,
            account_pubkey,
            segment_id,
            column_type,
            value_type,
        }
    }
}

impl fmt::Debug for ColumnHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ColumnHeader")
            .field("name", &self.name())
            .field("id", &self.id())
            .field("value_type", &self.value_type())
            .field("segment_id", &self.segment_id())
            .field("column_type", &self.column_type())
            .finish()
    }
}
