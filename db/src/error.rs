use account_fs::FSError;
use slice_rbtree::Error as RBTreeError;
use solana_program::program_error::ProgramError;

/// enum of possible errors
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    /// Error in account [FS](account_fs::FS)
    FSError(FSError),
    /// No columns left in the [`DB`](crate::DB) header
    NoColumnsLeft,
    /// Error in the [`RBTree`](slice_rbtree::tree::RBTree)
    RBTreeError(RBTreeError),
    /// The given segment is not a valid [`DB`](crate::DB) header
    WrongSegment,
    /// There are no column with such [`ColumnId`](crate::ColumnId)
    NoSuchColumn,
    /// This error occurs, than set_value_secondary() is called on secondary key which does not have a corresponding primary key
    SecondaryKeyWithNonExistentPrimaryKey,
    /// This error occurs, than not all columns are accessible during DB deletion
    NotAllColumnsArePresent,
    /// A column, used as a secondary key, must contain only unique key-value pairs
    NonUniqueSecondaryKey,
}

impl From<FSError> for Error {
    fn from(err: FSError) -> Self {
        Self::FSError(err)
    }
}

impl From<RBTreeError> for Error {
    fn from(err: RBTreeError) -> Self {
        Self::RBTreeError(err)
    }
}

const DB_IDENT: u8 = 0xDB;

impl From<Error> for ProgramError {
    fn from(err: Error) -> Self {
        use Error::*;
        let errno: u16 = match err {
            FSError(fs_err) => (1 << 8) + fs_err as u16,
            NoColumnsLeft => 2,
            RBTreeError(rb_err) => (3 << 8) + rb_err as u16,
            WrongSegment => 4,
            NoSuchColumn => 5,
            SecondaryKeyWithNonExistentPrimaryKey => 6,
            NotAllColumnsArePresent => 7,
            NonUniqueSecondaryKey => 8,
        };

        let error_code = ((DB_IDENT as u32) << 24) + errno as u32;
        ProgramError::Custom(error_code)
    }
}
