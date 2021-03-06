use account_fs::FSError;
use slice_rbtree::Error as RBTreeError;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    FSError(FSError),
    NoColumnsLeft,
    RBTreeError(RBTreeError),
    WrongSegment,
    NoSuchColumn,
    // This error occurs, than set_value_secondary() is called on secondary key which does not have
    // a corresponding primary key
    SecondaryKeyWithNonExistentPrimaryKey,
    // It happens, if not all columns are accessible during DB deletion
    NotAllColumnsArePresent,
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
