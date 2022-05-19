use borsh::BorshDeserialize;
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use std::fmt;

use account_fs::SegmentId;
use solcery_data_types::db::schema::{ColumnType, DataType};

#[repr(C)]
#[derive(Pod, Clone, Copy, Zeroable)]
pub struct ColumnHeader {
    name: [u8; 64],
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
        match self.value_type {
            0 => DataType::Int,
            1 => DataType::Float,
            3 => DataType::Pubkey,
            4 => DataType::ShortString,
            5 => DataType::MediumString,
            6 => DataType::LongString,
            _ => unreachable!("Unknown value type, it is a sign of data corruption"),
        }
    }

    pub fn segment_id(&self) -> SegmentId {
        SegmentId {
            pubkey: Pubkey::new_from_array(self.account_pubkey),
            id: u32::from_be_bytes(self.segment_id),
        }
    }

    pub fn column_type(&self) -> ColumnType {
        match self.value_type {
            0 => ColumnType::RBTree,
            _ => unreachable!("Unknown column type, it is a sign of data corruption"),
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
