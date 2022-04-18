#![deny(unsafe_op_in_unsafe_fn)]
#![feature(generic_const_exprs)]
#![allow(const_evaluatable_unchecked)]

use borsh::{BorshDeserialize, BorshSerialize};
use std::borrow::Borrow;
use std::cmp::Ord;

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
    WrongNodePoolSize,
    WrongRootsNumber,
    WrongValueSize,
}
