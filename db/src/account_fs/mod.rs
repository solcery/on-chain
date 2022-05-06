//! SolceryDB Account Filesystem
//!
//! This module manages data layout inside each account in the DB.
//!
//! Each account used in the DB has the following layout:
//!
//! First 33 bytes contain [AllocationTable](account_allocator::AllocationTable) struct, then goes
//! [Inode](account_allocator::Inode) table with `inodes_max` elements. Size of each Inode is 13
//! bytes. All the remaining space is usable for data.

mod account_allocator;
mod data_allocator;

pub use account_allocator::AccountAllocator;
pub use data_allocator::DataAllocator;
