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

impl From<Error> for ProgramError {
    fn from(err: Error) -> Self {
        todo!("Error conversion");
    }
}
