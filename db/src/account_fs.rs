//! SolceryDB Account Filesystem
//!
//! This module manages data layout inside each account in the DB.

use bytemuck::{cast_mut, cast_slice_mut};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use std::cell::RefMut;
use std::mem;
use tinyvec::SliceVec;

mod allocation_table;
mod inode;

use allocation_table::AllocationTable;
use inode::Inode;

#[derive(Debug)]
struct AccountAllocator<'short> {
    pubkey: Pubkey,
    free_chunks: Vec<&'short mut [u8]>,
    inode_data: SliceVec<'short, Inode>,
    allocation_table: &'short mut AllocationTable,
}

impl<'short, 'long: 'short> AccountAllocator<'short> {
    pub unsafe fn init_account(
        data: &'long mut [u8],
        max_inodes: usize,
        pubkey: Pubkey,
    ) -> Result<Self, Error> {
        let mut account_data = data;

        if account_data.len() < mem::size_of::<AllocationTable>() {
            return Err(Error::TooSmall);
        }
        let (allocation_table, tail): (&'short mut [u8], &'short mut [u8]) =
            account_data.split_at_mut(mem::size_of::<AllocationTable>());

        if tail.len() < max_inodes * mem::size_of::<Inode>() {
            return Err(Error::TooSmall);
        }
        let (inodes_slice, data_pool) = tail.split_at_mut(max_inodes * mem::size_of::<Inode>());

        let inodes: &mut [Inode] = cast_slice_mut(inodes_slice);
        let mut inode_data = SliceVec::from_slice_len(inodes, 0);

        let allocation_table: &mut [[u8; mem::size_of::<AllocationTable>()]] =
            cast_slice_mut(allocation_table);
        let allocation_table: &mut AllocationTable = cast_mut(&mut allocation_table[0]);

        unsafe {
            allocation_table.fill(max_inodes);

            let inode = Inode::from_raw_parts(0, data_pool.len(), None);
            inode_data.push(inode);
        }

        let free_chunks = vec![data_pool];

        Ok(Self {
            allocation_table,
            free_chunks,
            inode_data,
            pubkey,
        })
    }
}

pub enum Error {
    TooSmall,
}
