use bytemuck::{cast_mut, cast_slice_mut};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::mem;
use std::ptr::NonNull;
use std::slice::{from_raw_parts, from_raw_parts_mut};
use tinyvec::SliceVec;

mod allocation_table;
mod inode;

#[cfg(not(doc))]
use allocation_table::AllocationTable;
#[cfg(not(doc))]
use inode::Inode;

#[cfg(doc)]
pub use allocation_table::AllocationTable;
#[cfg(doc)]
pub use inode::Inode;

pub struct AccountAllocator<'long> {
    ptr: NonNull<u8>,
    len: usize,
    inode_data: SliceVec<'long, Inode>,
    allocation_table: &'long mut AllocationTable,
    borrowed_segments: BTreeSet<u32>,
    _ghost: PhantomData<&'long mut [u8]>,
}

impl<'long: 'short, 'short> AccountAllocator<'long> {
    pub unsafe fn init_account(data: &'long mut [u8], max_inodes: usize) -> Result<Self, Error> {
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

        let len = data.len();
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();

        let allocator = Self {
            allocation_table,
            borrowed_segments: BTreeSet::new(),
            inode_data,
            len,
            ptr,
            _ghost: PhantomData::<&'long mut [u8]>,
        };

        debug_assert!(allocator.is_ordered());

        Ok(allocator)
    }

    pub unsafe fn from_account(account_data: &'long mut [u8]) -> Result<Self, Error> {
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
            return Err(Error::WrongSize);
        }
        let (inodes_slice, data) =
            tail.split_at_mut(allocation_table.inodes_max() * mem::size_of::<Inode>());

        let inodes: &mut [Inode] = cast_slice_mut(inodes_slice);
        let inode_data = SliceVec::from_slice_len(inodes, allocation_table.inodes_count());

        let len = data.len();
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();

        let allocator = Self {
            allocation_table,
            borrowed_segments: BTreeSet::new(),
            inode_data,
            len,
            ptr,
            _ghost: PhantomData::<&'long mut [u8]>,
        };

        debug_assert!(allocator.is_ordered());

        Ok(allocator)
    }

    pub fn is_initialized(account_data: &'long mut [u8]) -> bool {
        if account_data.len() < mem::size_of::<AllocationTable>() {
            return false;
        }
        let (allocation_table, tail): (&'long mut [u8], &'long mut [u8]) =
            account_data.split_at_mut(mem::size_of::<AllocationTable>());

        let allocation_table: &mut [[u8; mem::size_of::<AllocationTable>()]] =
            cast_slice_mut(allocation_table);
        let allocation_table: &mut AllocationTable = cast_mut(&mut allocation_table[0]);

        if !allocation_table.check_magic() {
            return false;
        }

        if tail.len() < allocation_table.inodes_max() * mem::size_of::<Inode>() {
            return false;
        }
        let (inodes_slice, data) =
            tail.split_at_mut(allocation_table.inodes_max() * mem::size_of::<Inode>());

        let inodes: &mut [Inode] = cast_slice_mut(inodes_slice);
        let inode_data = SliceVec::from_slice_len(inodes, allocation_table.inodes_count());

        let len = data.len();
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();

        let allocator = Self {
            allocation_table,
            borrowed_segments: BTreeSet::new(),
            inode_data,
            len,
            ptr,
            _ghost: PhantomData::<&'long mut [u8]>,
        };

        allocator.is_ordered()
    }

    pub fn allocate_segment(&mut self, size: usize) -> Result<u32, Error> {
        if self.inode_data.len() == self.inode_data.capacity() {
            return Err(Error::NoInodesLeft);
        }

        let maybe_index = self
            .inode_data
            .iter()
            .enumerate()
            .find(|(_, inode)| inode.len() >= size && !inode.is_occupied())
            .map(|(index, _)| index);

        if let Some(index) = maybe_index {
            let inode = &mut self.inode_data[index];
            let start = inode.start_idx();
            let end = inode.end_idx();

            let id = self.allocation_table.generate_id();

            unsafe {
                if inode.len() == size {
                    inode.occupy(id);
                } else {
                    self.inode_data.swap_remove(index);
                    //TODO: reimplement without copying
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
            debug_assert!(self.is_ordered());

            Ok(id)
        } else {
            Err(Error::NoSuitableSegmentFound)
        }
    }

    pub fn deallocate_segment(&mut self, id: u32) -> Result<(), Error> {
        if self.borrowed_segments.contains(&id) {
            return Err(Error::Borrowed);
        }

        let result = self
            .inode_data
            .iter_mut()
            .find(|inode| inode.id() == Some(id))
            .map(|inode| inode.unoccupy())
            .ok_or(Error::NoSuchIndex);

        debug_assert_eq!(self.allocation_table.inodes_count(), self.inode_data.len());
        debug_assert!(self.is_ordered());

        result
    }

    pub fn segment(&mut self, id: u32) -> Result<&'short mut [u8], Error> {
        debug_assert!(self.is_ordered());
        if self.borrowed_segments.contains(&id) {
            return Err(Error::AlreadyBorrowed);
        }

        let maybe_inode = self
            .inode_data
            .iter()
            .find(|inode| inode.id() == Some(id) && inode.is_occupied());
        if let Some(inode) = maybe_inode {
            self.borrowed_segments.insert(id);

            unsafe {
                // Safety
                //
                // Safety contract of `from_raw_parts_mut`
                // * `data` must be valid for both reads and writes for `len * mem::size_of::<T>()` many bytes,
                //   and it must be properly aligned. This means in particular:
                //   -- Check: the emitted slice has the lifetime 'short which is not longer than 'long.
                //
                //     * The entire memory range of this slice must be contained within a single allocated object!
                //       Slices can never span across multiple allocated objects.
                //       -- Check: we are offsetting inside one big slice.
                //
                //     * `data` must be non-null and aligned even for zero-length slices. One
                //       reason for this is that enum layout optimizations may rely on references
                //       (including slices of any length) being aligned and non-null to distinguish
                //       them from other data. You can obtain a pointer that is usable as `data`
                //       for zero-length slices using [`NonNull::dangling()`].
                //       -- Check: data_ptr is not null and so does slice_ptr.
                //
                // * `data` must point to `len` consecutive properly initialized values of type `T`.
                //    -- Check: the original slice of bytes was initialized,
                //       offset bounds was checked to be inside the original slice.
                //
                // * The memory referenced by the returned slice must not be accessed through any other pointer
                //   (not derived from the return value) for the duration of lifetime `'short`.
                //   Both read and write accesses are forbidden.
                //   -- Check: self.borrowed_segments guarantees, that this segment will not be emitted again.
                //      Segments was checked to be non-overlapping.
                //
                // * The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`.
                //   -- Check: here we are limited by the maximum account size, which is far less than `isize::MAX`.
                let offset_start = inode.start_idx();
                let offset_end = inode.end_idx();

                debug_assert!(offset_end <= self.len);
                debug_assert!(offset_start < offset_end);

                let data_ptr = self.ptr.as_ptr();
                let len = offset_end - offset_start;
                let slice_ptr = data_ptr.add(offset_start);

                Ok(from_raw_parts_mut(slice_ptr, len))
            }
        } else {
            Err(Error::NoSuchSegment)
        }
    }

    pub fn merge_segments(&mut self) {
        unimplemented!();
    }

    /// Marks a segment as unborrowed
    ///
    /// # Safety
    /// The caller must assert, that all the slices pointing to this segment are dropped
    pub unsafe fn release_borrowed_segment(&mut self, id: u32) {
        self.borrowed_segments.remove(&id);
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

        if self.inode_data[self.inode_data.len() - 1].end_idx() != self.len {
            return false;
        }

        true
    }

    fn collect_slices(&self) -> BTreeMap<u32, &[u8]> {
        if !self.borrowed_segments.is_empty() {
            panic!("Can't compare AccountAllocators with borrowed segments");
        }

        self.inode_data
            .iter()
            .filter_map(|inode| {
                if let Some(id) = inode.id() {
                    unsafe {
                        // Safety
                        //
                        // Safety contract of `from_raw_parts_mut`
                        // * `data` must be valid for both reads and writes for `len * mem::size_of::<T>()` many bytes,
                        //   and it must be properly aligned. This means in particular:
                        //   -- Check: the emitted slice has the lifetime 'short which is not longer than 'long.
                        //
                        //     * The entire memory range of this slice must be contained within a single allocated object!
                        //       Slices can never span across multiple allocated objects.
                        //       -- Check: we are offsetting inside one big slice.
                        //
                        //     * `data` must be non-null and aligned even for zero-length slices. One
                        //       reason for this is that enum layout optimizations may rely on references
                        //       (including slices of any length) being aligned and non-null to distinguish
                        //       them from other data. You can obtain a pointer that is usable as `data`
                        //       for zero-length slices using [`NonNull::dangling()`].
                        //       -- Check: data_ptr is not null and so does slice_ptr.
                        //
                        // * `data` must point to `len` consecutive properly initialized values of type `T`.
                        //    -- Check: the original slice of bytes was initialized,
                        //       offset bounds was checked to be inside the original slice.
                        //
                        // * The memory referenced by the returned slice must not be accessed through any other pointer
                        //   (not derived from the return value) for the duration of lifetime `'short`.
                        //   Both read and write accesses are forbidden.
                        //   -- Check: self.borrowed_segments guarantees, that this segment will not be emitted again.
                        //      Segments was checked to be non-overlapping.
                        //
                        // * The total size `len * mem::size_of::<T>()` of the slice must be no larger than `isize::MAX`.
                        //   -- Check: here we are limited by the maximum account size, which is far less than `isize::MAX`.
                        let offset_start = inode.start_idx();
                        let offset_end = inode.end_idx();

                        debug_assert!(offset_end <= self.len);
                        debug_assert!(offset_start < offset_end);

                        let data_ptr = self.ptr.as_ptr();
                        let len = offset_end - offset_start;
                        let slice_ptr = data_ptr.add(offset_start);

                        Some((id, from_raw_parts(slice_ptr, len)))
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    #[cfg(test)]
    pub(super) fn account_size(inode_count: usize, data_size: usize) -> usize {
        mem::size_of::<AllocationTable>() + inode_count * mem::size_of::<Inode>() + data_size
    }
}

impl<'a> fmt::Debug for AccountAllocator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AccountAllocator")
            .field("allocation_table", &self.allocation_table)
            .field("data_size", &self.len)
            .field("inodes", &self.inode_data)
            .finish()
    }
}

impl<'a> Eq for AccountAllocator<'a> {}

impl<'a> PartialEq for AccountAllocator<'a> {
    fn eq(&self, other: &Self) -> bool {
        let lhs_segments = self.collect_slices();

        let rhs_segments = other.collect_slices();

        for (lhs, rhs) in lhs_segments.iter().zip(rhs_segments.iter()) {
            let (lhs_id, lhs_slice) = lhs;
            let (rhs_id, rhs_slice) = rhs;

            if lhs_id != rhs_id {
                return false;
            }
            if lhs_slice != rhs_slice {
                return false;
            }
        }
        true
    }
}

/// The reasons, why FS operations may fail.
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum Error {
    /// attempted to borrow one segment twice
    AlreadyBorrowed,
    /// attempted to deallocate borrowed segment
    Borrowed,
    /// There are no free inodes left in the account
    NoInodesLeft,
    /// There are no segments with the given id in the given account
    NoSuchIndex,
    /// There are no accounts in the [`FS`](super::FS) with the supplied [`Pubkey`](super::Pubkey)
    NoSuchPubkey,
    /// There is segment with the supplied [`SegmentId`](super::SegmentId)
    NoSuchSegment,
    /// There are no continuous area in the account, that can be allocated
    ///
    /// Try to call [`defragment`](super::FS::defragment)
    NoSuitableSegmentFound,
    /// The account is too small for [`FS`](super::FS) internal structures
    TooSmall,
    /// [`FS`](super::FS) header has incorrect magic, maybe it is not initialized?
    WrongMagic,
    /// The size of [`FS`](super::FS) in the header does no match with the actual account size
    WrongSize,
    /// The account owner does not match `program_id`, such account can not be used as part of [`FS`](super::FS)
    WrongOwner,
}

#[cfg(test)]
mod tests;
