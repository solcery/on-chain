#![deny(unsafe_op_in_unsafe_fn)]

mod forest;
mod tree;

pub use forest::RBForest;
use forest::{KeysIterator, PairsIterator, ValuesIterator};
pub use tree::RBTree;

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
