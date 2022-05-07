//! SolceryDB Account Filesystem
//!
//! This module manages data layout inside each account in the DB.
//!
//! Each account used in the DB has the following layout:
//!
//! First 33 bytes contain [AllocationTable](account_allocator::AllocationTable) struct, then goes
//! [Inode](account_allocator::Inode) table with `inodes_max` elements. Size of each Inode is 13
//! bytes. All the remaining space is usable for data.

#![deny(unsafe_op_in_unsafe_fn)]
#![feature(cell_leak)]
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, program_error::ProgramError,
    pubkey::Pubkey,
};
use std::cell::RefMut;
use std::collections::BTreeMap;

mod account_allocator;
mod data_allocator;
mod segment_id;

pub use account_allocator::{AccountAllocator, Error as AllocatorError};
pub use data_allocator::DataAllocator;
pub use segment_id::SegmentId;

#[derive(Debug)]
pub struct FSAllocator<'a> {
    allocators: BTreeMap<Pubkey, AccountAllocator<'a>>,
}

impl<'a> FSAllocator<'a> {
    /// Constructs [FSAllocator], assuming that all account are initialized as filesystem accounts
    pub fn from_account_iter<AccountIter>(
        accounts_iter: &mut AccountIter,
    ) -> Result<Self, AllocatorError>
    where
        AccountIter: Iterator<Item = &'a AccountInfo<'a>>,
    {
        let result: Result<BTreeMap<Pubkey, AccountAllocator<'a>>, _> = accounts_iter
            .map(|account| {
                let pubkey = *account.key;
                let data = account.data.borrow_mut();
                let data = RefMut::<'_, &'a mut [u8]>::leak(data);
                unsafe { AccountAllocator::from_account(data).map(|alloc| (pubkey, alloc)) }
            })
            .collect();

        result.map(|allocators| Self { allocators })
    }

    /// Constructs [FSAllocator], assuming that some (or any) accounts may be uninitialized as filesystem accounts
    pub fn from_uninit_account_iter<AccountIter>(
        accounts_iter: &mut AccountIter,
    ) -> Result<Self, AllocatorError>
    where
        AccountIter: Iterator<Item = &'a AccountInfo<'a>>,
    {
        //let result: Result<BTreeMap<Pubkey, AccountAllocator<'a>>, _> = accounts_iter
        //.map(|account| {
        //let pubkey = *account.key;
        //let data = account.data.borrow_mut();
        //let data = RefMut::<'_, &'a mut [u8]>::leak(data);
        //let maybe_alloc = unsafe { AccountAllocator::from_account(*data) };

        //// We can't use match here because of borrow checker
        //if let Ok(alloc) = maybe_alloc {
        //return Ok((pubkey, alloc));
        //} else if maybe_alloc.unwrap_err() == AllocatorError::TooSmall {
        //Err(AllocatorError::TooSmall)
        //} else {
        //unsafe { AccountAllocator::from_account(*data).map(|alloc| (pubkey, alloc)) }
        //}
        //})
        //.collect();

        //result.map(|allocators| Self { allocators })
        unimplemented!(); // Thanks to Borrow Checker
    }
}
