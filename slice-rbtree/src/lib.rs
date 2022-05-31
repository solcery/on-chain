#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]

mod forest;
mod tree;

pub use forest::{forest_size, RBForest};
use forest::{KeysIterator, PairsIterator, ValuesIterator};
pub use tree::{tree_size, RBTree};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    KeySerializationError,
    NoNodesLeft,
    TooSmall,
    ValueSerializationError,
    WrongKeySize,
    WrongMagic,
    WrongNodePoolSize,
    WrongRootsNumber,
    WrongValueSize,
}
