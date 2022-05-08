//! SolceryDB Account Filesystem
//!
//! This module manages data layout inside each account in the DB.
//!
//! Each account used in the DB has the following layout:
//!
//! * First 33 bytes contain [AllocationTable](account_allocator::AllocationTable) struct
//! * then goes [Inode](account_allocator::Inode) table with `inodes_max` elements. Size of each
//! Inode is 13 bytes.
//! * All the remaining space is usable for data.
//!
//! Operation with FS is splitted into two stages:
//! - First stage is represented by [FSAllocator]. At this stage you can allocate and deallocate
//! segments of memory.
//! - Second stage is represented by [FSDispatcher], which can only be condtructed from
//! [FSAllocator]. At this stage you can get mutable slices to allocated memory segments.

#![deny(unsafe_op_in_unsafe_fn)]
#![feature(cell_leak)]

mod account_allocator;
mod data_allocator;
mod fs_allocator;
mod fs_dispatcher;
mod segment_id;

use account_allocator::AccountAllocator;
use data_allocator::DataAllocator;

pub use account_allocator::Error as AllocatorError;
pub use data_allocator::DataError;
pub use fs_allocator::FSAllocator;
pub use fs_dispatcher::FSDispatcher;
pub use segment_id::SegmentId;
