//! A `#[no_std]` Red-Black tree, fully packed in a single slice
//!
//! Originally developed for storing data in [Solana][0] [Accounts][1], this crate allows you to
//! access tree nodes without deserializing the whole tree. It is useful, then you have a huge
//! tree in raw memory, but want to interact only with a few values at a time.
//!
//! [0]: https://docs.solana.com/
//! [1]: https://docs.rs/solana-sdk/latest/solana_sdk/account/struct.Account.html
//!
//! # A  small example
//! Let's assume, you want to create a tree, holding up to 100 pairs of `u8 <-> f64`:
//! ```
//! use slice_rbtree::{tree_size, RBTree};
//! // RBTree requires input slice to have a proper size
//! // 1 == size_of::<u8>(), 8 == size_of::<f64>()
//! let size = tree_size(1, 8, 100);
//! let mut buffer = vec![0; size];
//!
//! let mut tree: RBTree<u8, f64, 1, 8> = RBTree::init_slice(&mut buffer).unwrap();
//!
//! tree.insert(15, 1.245).unwrap();
//!
//! drop(tree);
//!
//! let new_tree: RBTree<u8, f64, 1, 8> = unsafe { RBTree::from_slice(&mut buffer).unwrap() };
//! assert_eq!(new_tree.get(&15), Some(1.245));
//! ```
//!
// # Benchmarks
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
#![cfg_attr(not(test), no_std)]

use borsh::{BorshDeserialize, BorshSerialize};
mod forest;
mod tree;

pub use forest::{forest_size, init_forest, RBForest};
#[doc(hidden)]
pub use forest::{KeysIterator, PairsIterator, ValuesIterator};
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
