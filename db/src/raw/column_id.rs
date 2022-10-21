use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;

/// Opaque column identifier
#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct ColumnId(u32);

impl ColumnId {
    /// Generates [`ColumnId`] from [`u32`]
    pub fn new(val: u32) -> Self {
        Self(val)
    }

    pub(crate) fn to_be_bytes(self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}

impl fmt::Debug for ColumnId {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let tmp = self.0.to_string();

        formatter.pad_integral(true, "", &tmp)
    }
}
