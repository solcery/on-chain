use account_fs::FSError;
use slice_rbtree::Error as RBTreeError;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum Error {
    FSError(FSError),
    NoColumnsLeft,
    RBTreeError(RBTreeError),
    WrongSegment,
    NoSuchColumn,
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
