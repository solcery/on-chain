#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
#[repr(transparent)]
pub struct ColumnId(u32);

impl ColumnId {
    pub(crate) fn new(val: u32) -> Self {
        Self(val)
    }

    pub(crate) fn to_be_bytes(&self) -> [u8; 4] {
        self.0.to_be_bytes()
    }
}
