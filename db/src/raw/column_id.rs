use borsh::{BorshDeserialize, BorshSerialize};
use std::fmt;

#[derive(BorshDeserialize, BorshSerialize, Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct ColumnId(u32);

impl ColumnId {
    pub(crate) fn new(val: u32) -> Self {
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
