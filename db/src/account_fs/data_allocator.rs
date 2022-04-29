use core::slice::from_raw_parts_mut;
use solana_program::pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::marker::PhantomData;
use std::ptr::NonNull;

type Segment = (usize, usize);

pub struct DataAllocator<'long> {
    pubkey: Pubkey,
    ptr: NonNull<u8>,
    len: usize,
    allocated_segments: BTreeMap<u32, Segment>,
    borrowed_serments: BTreeSet<u32>,
    _ghost: PhantomData<&'long mut [u8]>,
}

impl<'long> fmt::Debug for DataAllocator<'long> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataAllocator")
            .field("pubkey", &self.pubkey)
            .field("allocated_segments", &self.allocated_segments)
            .field("borrowed_serments", &self.borrowed_serments)
            .field("data_size", &self.len)
            .finish()
    }
}

impl<'long: 'short, 'short> DataAllocator<'long> {
    pub fn segment(&mut self, id: u32) -> Result<&'short mut [u8], DataError> {
        if self.borrowed_serments.contains(&id) {
            return Err(DataError::AlreadyBorrowed);
        }

        if let Some(&(offset_start, offset_end)) = self.allocated_segments.get(&id) {
            self.borrowed_serments.insert(id);

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

                debug_assert!(offset_end <= self.len);
                debug_assert!(offset_start < offset_end);

                let data_ptr = self.ptr.as_ptr();
                let len = offset_end - offset_start;
                let slice_ptr = data_ptr.add(offset_start);

                Ok(from_raw_parts_mut(slice_ptr, len))
            }
        } else {
            Err(DataError::NoSuchSegment)
        }
    }

    pub(super) unsafe fn from_raw_parts(
        pubkey: Pubkey,
        data: &'long mut [u8],
        allocated_segments: BTreeMap<u32, Segment>,
        borrowed_serments: BTreeSet<u32>,
    ) -> Self {
        let len = data.len();
        let ptr = NonNull::new(data.as_mut_ptr()).unwrap();
        Self {
            pubkey,
            len,
            ptr,
            allocated_segments,
            borrowed_serments,
            _ghost: PhantomData::<&'long mut [u8]>,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum DataError {
    NoSuchSegment,
    AlreadyBorrowed,
}

#[cfg(test)]
use super::AccountAllocator;
#[cfg(test)]
mod tests;
