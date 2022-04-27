use super::DataAllocator;
use bytemuck::{cast_mut, cast_slice_mut};
use solana_program::pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::mem;
use tinyvec::SliceVec;
mod allocation_table;
mod inode;

use allocation_table::AllocationTable;
use inode::Inode;

pub struct AccountAllocator<'long> {
    pubkey: Pubkey,
    data: &'long mut [u8],
    inode_data: SliceVec<'long, Inode>,
    allocation_table: &'long mut AllocationTable,
}

impl<'long> AccountAllocator<'long> {
    pub unsafe fn init_account(
        data: &'long mut [u8],
        max_inodes: usize,
        pubkey: Pubkey,
    ) -> Result<Self, Error> {
        let account_data = data;

        if account_data.len() < mem::size_of::<AllocationTable>() {
            return Err(Error::TooSmall);
        }
        let (allocation_table, tail): (&'long mut [u8], &'long mut [u8]) =
            account_data.split_at_mut(mem::size_of::<AllocationTable>());

        if tail.len() < max_inodes * mem::size_of::<Inode>() {
            return Err(Error::TooSmall);
        }
        let (inodes_slice, data) = tail.split_at_mut(max_inodes * mem::size_of::<Inode>());

        let inodes: &mut [Inode] = cast_slice_mut(inodes_slice);
        let mut inode_data = SliceVec::from_slice_len(inodes, 0);

        let allocation_table: &mut [[u8; mem::size_of::<AllocationTable>()]] =
            cast_slice_mut(allocation_table);
        let allocation_table: &mut AllocationTable = cast_mut(&mut allocation_table[0]);

        unsafe {
            allocation_table.fill(max_inodes);

            let inode = Inode::from_raw_parts(0, data.len(), None);
            inode_data.push(inode);
        }

        let allocator = Self {
            allocation_table,
            data,
            inode_data,
            pubkey,
        };

        debug_assert!(allocator.is_ordered());

        Ok(allocator)
    }

    pub unsafe fn from_account(
        account_data: &'long mut [u8],
        pubkey: Pubkey,
    ) -> Result<Self, Error> {
        if account_data.len() < mem::size_of::<AllocationTable>() {
            return Err(Error::TooSmall);
        }
        let (allocation_table, tail): (&'long mut [u8], &'long mut [u8]) =
            account_data.split_at_mut(mem::size_of::<AllocationTable>());

        let allocation_table: &mut [[u8; mem::size_of::<AllocationTable>()]] =
            cast_slice_mut(allocation_table);
        let allocation_table: &mut AllocationTable = cast_mut(&mut allocation_table[0]);

        if !allocation_table.check_magic() {
            return Err(Error::WrongMagic);
        }

        if tail.len() < allocation_table.inodes_max() * mem::size_of::<Inode>() {
            return Err(Error::TooSmall);
        }
        let (inodes_slice, data) =
            tail.split_at_mut(allocation_table.inodes_max() * mem::size_of::<Inode>());

        let inodes: &mut [Inode] = cast_slice_mut(inodes_slice);
        let inode_data = SliceVec::from_slice_len(inodes, allocation_table.inodes_count());

        let allocator = Self {
            allocation_table,
            data,
            inode_data,
            pubkey,
        };

        debug_assert!(allocator.is_ordered());

        Ok(allocator)
    }

    pub fn allocate_chunk(&mut self, size: usize) -> Result<u32, Error> {
        if self.inode_data.len() == self.inode_data.capacity() {
            return Err(Error::NoInodesLeft);
        }

        let maybe_index = self
            .inode_data
            .iter()
            .enumerate()
            .find(|(_, inode)| inode.len() >= size && !inode.occupied())
            .map(|(index, _)| index);

        if let Some(index) = maybe_index {
            let inode = &mut self.inode_data[index];
            let start = inode.start_idx();
            let end = inode.end_idx();

            let id = self.allocation_table.genereate_id();
            unsafe {
                if inode.len() == size {
                    inode.occupy(id);
                } else {
                    self.inode_data.swap_remove(index);
                    //TODO: reimplement wihout copying
                    let new_inode1 = Inode::from_raw_parts(start, start + size, Some(id));
                    let new_inode2 = Inode::from_raw_parts(start + size, end, None);

                    self.inode_data.push(new_inode1);
                    self.inode_data.push(new_inode2);
                    self.inode_data.sort_by_key(|inode| inode.start_idx());

                    self.allocation_table
                        .set_inodes_count(self.inode_data.len());
                }
            }
            debug_assert_eq!(self.allocation_table.inodes_count(), self.inode_data.len());
            Ok(id)
        } else {
            Err(Error::NoSuitableChunkFound)
        }
    }

    pub fn deallocate_chunk(&mut self, id: u32) -> Result<(), Error> {
        let result = self
            .inode_data
            .iter_mut()
            .find(|inode| inode.id() == Some(id))
            .map(|inode| inode.unoccupy())
            .ok_or(Error::NoSuchIndex);

        debug_assert_eq!(self.allocation_table.inodes_count(), self.inode_data.len());

        result
    }

    pub fn to_data_allocator(self) -> DataAllocator<'long> {
        let allocated_segments = BTreeMap::<u32, (usize, usize)>::from_iter(
            self.inode_data
                .iter()
                .filter(|inode| inode.occupied())
                // We have already filtered all unoccupied inodes, so unwrap will always succeed
                .map(|inode| (inode.id().unwrap(), (inode.start_idx(), inode.end_idx()))),
        );
        let borrowed_serments = BTreeSet::new();
        let data = self.data;
        let pubkey = self.pubkey;

        unsafe {
            DataAllocator::from_raw_parts(pubkey, data, allocated_segments, borrowed_serments)
        }
    }

    fn is_ordered(&self) -> bool {
        if self.inode_data[0].start_idx() != 0 {
            return false;
        }

        for arr in self.inode_data.windows(2) {
            let first = arr[0];
            let second = arr[1];
            if first.end_idx() != second.start_idx() {
                return false;
            }
        }

        if self.inode_data[self.inode_data.len() - 1].end_idx() != self.data.len() {
            return false;
        }
        return true;
    }
}

impl<'a> fmt::Debug for AccountAllocator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountAllocator")
            .field("pubkey", &self.pubkey)
            .field("allocation_table", &self.allocation_table)
            .field("data_size", &self.data.len())
            .field("inodes", &self.inode_data)
            .finish()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    TooSmall,
    WrongMagic,
    NoInodesLeft,
    NoSuitableChunkFound,
    NoSuchIndex,
}

#[cfg(test)]
mod tests;
