#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

use borsh::{BorshDeserialize, BorshSerialize};
mod forest;
mod tree;

pub use forest::{forest_size, init_forest, RBForest};
use forest::{KeysIterator, PairsIterator, ValuesIterator};
pub use tree::{init_tree, tree_size, RBTree};

#[derive(Debug, PartialEq, Eq, Copy, Clone, BorshDeserialize, BorshSerialize)]
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
