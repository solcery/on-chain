//! # Solcery Account Filesystem
//!
//! This crate manages data layout inside accounts.
//! The idea is to work with a set of accounts as an abstract allocator ("file system") wich can
//! allocate and deallocate slices of bytes.
//!
//! # Minimal example
//! We are in a program with some `program_id` and want to initialize [`FS`] in a set of accounts,
//! owned by this program. This accounts are in `accounts_iter` iterator.
//!
//! When initializing accounts, we have to say, into how much chunks we want to split each account.
//! We neeed to do it, because it dictates, how much space must be allocated for internal
//! structures.
//!
//!```rust
//! # use fs_test::*;
//! use account_fs::FS;
//! # use solana_program::pubkey::Pubkey;
//!
//! # let program_id = Pubkey::new_unique();
//! # let params = AccountParams {
//! #     address: None,
//! #     owner: program_id,
//! #     data: AccountData::Empty(10_000),
//! # };
//! let mut fs_data = FSAccounts::replicate_params(params, 3);
//! let account_infos = fs_data.account_info_iter();
//!
//! # let mut accounts_iter = account_infos.iter();
//! // ... somehow obtained program_id and accounts_iter
//!
//!// Build FS with account initialization
//! let mut fs =
//!     FS::from_uninit_account_iter(
//!         &program_id,
//!         &mut accounts_iter,
//!         10 // This is the maximum number of segments in each account
//!     ).unwrap();
//!
//! // Allocate a segment of 150 bytes
//! let segment_id = fs.allocate_segment(150).unwrap();
//!
//! // work with segments
//! {
//!     // Borrow previously allocated segment
//!     let segment: &mut [u8] = fs.segment(&segment_id).unwrap();
//!
//!     // Attempt to borrow the segment for the second time will fail
//!     fs.segment(&segment_id).unwrap_err();
//!     // do stuff
//!     segment[0] = 10;
//!     segment[15] = 118;
//! }
//!
//! // The borrow of the segment is dropped, so it is safe to mark the segment as not borrowed
//! unsafe {
//!     // This function is unsafe, because the absence of non-dropped borrows has to be checked manually.
//!     fs.release_borrowed_segment(&segment_id);
//! }
//!
//! // Check, that the values we've written are still there
//! let segment: &mut [u8] = fs.segment(&segment_id).unwrap();
//!
//! assert_eq!(segment[0], 10);
//! assert_eq!(segment[15], 118);
//!```
//! # Internal structure
//! Internally [`FS`] is set of [`AccountAllocators`](AccountAllocator). Build docs with
//! `--document-private-items` to see its documentation

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![feature(cell_leak)]
#![feature(slice_partition_dedup)]

use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

use std::cell::RefMut;
use std::collections::BTreeMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

mod account_allocator;
mod segment_id;

use account_allocator::AccountAllocator;

pub use account_allocator::Error as FSError;
pub use segment_id::SegmentId;

/// A struct which allocates and deallocates bytes
pub struct FS<'long: 'short, 'short> {
    allocators: BTreeMap<Pubkey, (AccountAllocator<'short>, &'short AccountInfo<'long>)>,
}

impl<'long: 'short, 'short> FS<'long, 'short> {
    /// Constructs [`FS`], assuming that all accounts are initialized as filesystem accounts
    pub fn from_account_iter<AccountIter>(
        program_id: &Pubkey,
        accounts_iter: &mut AccountIter,
    ) -> Result<Self, FSError>
    where
        AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
    {
        let result: Result<
            BTreeMap<Pubkey, (AccountAllocator<'short>, &'short AccountInfo<'long>)>,
            _,
        > = accounts_iter
            .map(|account| {
                if account.owner != program_id {
                    return Err(FSError::WrongOwner);
                }
                let pubkey = *account.key;
                let cell = account.data.borrow_mut();
                let data = RefMut::<'_, &'long mut [u8]>::leak(cell);
                unsafe {
                    AccountAllocator::from_account_unchecked(data)
                        .map(|alloc| (pubkey, (alloc, account)))
                }
            })
            .collect();

        result.map(|allocators| Self { allocators })
    }

    /// Constructs [`FS`], assuming that some (or all) accounts may be uninitialized as filesystem accounts
    pub fn from_uninit_account_iter<AccountIter>(
        program_id: &Pubkey,
        accounts_iter: &mut AccountIter,
        inode_table_size: usize,
    ) -> Result<Self, FSError>
    where
        AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
    {
        let result: Result<
            BTreeMap<Pubkey, (AccountAllocator<'short>, &'short AccountInfo<'long>)>,
            _,
        > = accounts_iter
            .map(|account| {
                if account.owner != program_id {
                    return Err(FSError::WrongOwner);
                }
                let pubkey = *account.key;
                let data = account.data.borrow_mut();
                let data = RefMut::<'_, &'long mut [u8]>::leak(data);
                if AccountAllocator::is_initialized(data) {
                    unsafe {
                        AccountAllocator::from_account_unchecked(data)
                            .map(|alloc| (pubkey, (alloc, account)))
                    }
                } else {
                    AccountAllocator::init_account(data, inode_table_size)
                        .map(|alloc| (pubkey, (alloc, account)))
                }
            })
            .collect();

        result.map(|allocators| Self { allocators })
    }

    /// Allocates segment of data in the first account with available space
    pub fn allocate_segment(&mut self, size: usize) -> Result<SegmentId, FSError> {
        use FSError::{NoInodesLeft, NoSuitableSegmentFound};

        let mut global_result = Err(NoSuitableSegmentFound);
        for (key, (alloc, _)) in self.allocators.iter_mut() {
            let allocation_result = alloc.allocate_segment(size);
            match (allocation_result, global_result) {
                (Ok(id), _) => {
                    return Ok(SegmentId { pubkey: *key, id });
                }
                (Err(NoSuitableSegmentFound), Err(NoInodesLeft)) => {
                    global_result = Err(NoSuitableSegmentFound);
                }
                (Err(_), Err(_)) => {}
                (Err(_), Ok(_)) => unreachable!(),
            }
        }
        global_result
    }

    /// Deallocates the segment with a given [`SegmentId`]
    ///
    /// Only unborrowed segments can be deallocated
    pub fn deallocate_segment(&mut self, id: &SegmentId) -> Result<(), FSError> {
        match self.allocators.get_mut(&id.pubkey) {
            Some((alloc, _)) => alloc.deallocate_segment(id.id),
            None => Err(FSError::NoSuchPubkey),
        }
    }

    /// Marks a segment as unborrowed
    ///
    /// # Safety
    /// The caller must assert, that all borrows pointing to this segment are dropped
    pub unsafe fn release_borrowed_segment(&mut self, id: &SegmentId) {
        if let Some((alloc, _)) = self.allocators.get_mut(&id.pubkey) {
            unsafe {
                alloc.release_borrowed_segment(id.id);
            }
        }
    }

    #[doc(hidden)]
    pub fn defragment(&mut self) {
        for (_, (alloc, _)) in self.allocators.iter_mut() {
            alloc.defragment();
        }
    }

    /// Borrows a segment with given [`SegmentId`]
    pub fn segment(&mut self, id: &SegmentId) -> Result<&'short mut [u8], FSError> {
        match self.allocators.get_mut(&id.pubkey) {
            Some((alloc, _)) => alloc.segment(id.id),
            None => Err(FSError::NoSuchPubkey),
        }
    }

    /// Checks if a segment with given [`SegmentId`] can be accessed in the FS
    ///
    /// [`SegmentId`] consists of [`Pubkey`] and `u32` id. This function checks, if account with the
    /// given [`Pubkey`] present in the [`FS`].
    pub fn is_accessible(&self, id: &SegmentId) -> bool {
        self.allocators.contains_key(&id.pubkey)
    }
}

impl<'long: 'short, 'short> Drop for FS<'long, 'short> {
    fn drop(&mut self) {
        for (_, account_info) in self.allocators.values_mut() {
            let cell = account_info.data.clone();
            unsafe {
                let ptr = Rc::into_raw(cell);
                Rc::decrement_strong_count(ptr);

                let mut ref_counter = Rc::from_raw(ptr);
                Rc::get_mut(&mut ref_counter)
                    .expect("Already borrowed")
                    .undo_leak();

                Rc::increment_strong_count(ptr);
            }
        }
    }
}

impl<'long: 'short, 'short> Debug for FS<'long, 'short> {
    fn fmt(&self, fmt: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_map()
            .entries(
                self.allocators
                    .iter()
                    .map(|(pubkey, (alloc, _))| (pubkey, alloc)),
            )
            .finish()
    }
}
