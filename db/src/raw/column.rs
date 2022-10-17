use borsh::{BorshDeserialize, BorshSerialize};
use bytemuck::{Pod, Zeroable};
use solana_program::pubkey::Pubkey;
use std::fmt;

use super::column_id::ColumnId;
use crate::{ColumnType, DataType};
use account_fs::SegmentId;

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

    pub fn id(&self) -> ColumnId {
        ColumnId::new(u32::from_be_bytes(self.id))
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
        ColumnType::try_from(self.column_type)
            .expect("Unknown column type, it is a sign of data corruption")
    }

    pub unsafe fn new(
        name: &str,
        id: ColumnId,
        segment_id: SegmentId,
        value_type: DataType,
        column_type: ColumnType,
    ) -> Self {
        assert!(name.len() <= NAME_LEN - 4); // 4 bytes will be used as string length inside
                                             // BorshSerialize
        let mut name_bytes: [u8; NAME_LEN] = [0; NAME_LEN];
        name.serialize(&mut name_bytes.as_mut_slice()).unwrap();
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

impl Default for ColumnHeader {
    fn default() -> Self {
        Self {
            name: [0; NAME_LEN],
            id: [0; 4],
            value_type: 0,
            account_pubkey: [0; 32],
            segment_id: [0; 4],
            column_type: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    #[should_panic]
    fn too_long_name() {
        let name = "An Entirely Too Long Name For Such A Small Column Which Would Obviously Be An Overkill";
        let id = ColumnId::new(0);
        let segment_id = SegmentId {
            pubkey: Pubkey::new_unique(),
            id: 0,
        };
        let value_type = DataType::Int;
        let column_type = ColumnType::RBTree;

        unsafe {
            ColumnHeader::new(name, id, segment_id, value_type, column_type);
        }
    }

    #[test]
    fn new() {
        let name = "Just Some Name";
        let id = ColumnId::new(12345);
        let segment_id = SegmentId {
            pubkey: Pubkey::new_unique(),
            id: 5,
        };
        let value_type = DataType::ShortString;
        let column_type = ColumnType::RBTree;

        let column = unsafe { ColumnHeader::new(name, id, segment_id, value_type, column_type) };

        assert_eq!(column.id(), id);
        assert_eq!(column.segment_id(), segment_id);
        assert_eq!(column.value_type(), value_type);
        assert_eq!(column.name(), name.to_string());
        assert_eq!(column.column_type(), column_type);
    }
}
