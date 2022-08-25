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

#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
#![feature(cell_leak)]

use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::BTreeMap;
use std::rc::Rc;

mod account_allocator;
mod segment_id;

use account_allocator::AccountAllocator;

pub use account_allocator::Error as FSError;
pub use segment_id::SegmentId;

#[derive(Debug)]
pub struct FS<'long: 'short, 'short> {
    allocators: BTreeMap<Pubkey, (AccountAllocator<'short>, &'short AccountInfo<'long>)>,
}

impl<'long: 'short, 'short> FS<'long, 'short> {
    /// Constructs [FS], assuming that all accounts are initialized as filesystem accounts
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
                    AccountAllocator::from_account(data).map(|alloc| (pubkey, (alloc, account)))
                }
            })
            .collect();

        result.map(|allocators| Self { allocators })
    }

    /// Constructs [FS], assuming that some (or all) accounts may be uninitialized as filesystem accounts
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
                        AccountAllocator::from_account(data).map(|alloc| (pubkey, (alloc, account)))
                    }
                } else {
                    unsafe {
                        AccountAllocator::init_account(data, inode_table_size)
                            .map(|alloc| (pubkey, (alloc, account)))
                    }
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

    /// Deallocates segment of data in the first account with available space
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

    pub fn defragment_fs(&mut self) {
        for (_, (alloc, _)) in self.allocators.iter_mut() {
            alloc.merge_segments();
        }
    }

    /// Borrows a segment with given [SegmentId]
    pub fn segment(&mut self, id: &SegmentId) -> Result<&'short mut [u8], FSError> {
        match self.allocators.get_mut(&id.pubkey) {
            Some((alloc, _)) => alloc.segment(id.id),
            None => Err(FSError::NoSuchPubkey),
        }
    }

    /// Checks if a segment with given [SegmentId] is present in the FS
    pub fn is_accessible(&self, id: &SegmentId) -> bool {
        self.allocators.contains_key(&id.pubkey)
    }
}

impl<'long: 'short, 'short> Drop for FS<'long, 'short> {
    fn drop(&mut self) {
        for (_, account_info) in self.allocators.values_mut() {
            let mut cell = account_info.data.clone();
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
