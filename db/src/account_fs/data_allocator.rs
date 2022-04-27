use super::AccountAllocator;
use core::slice::from_raw_parts_mut;
use solana_program::pubkey::Pubkey;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

type SegmentOffset = (usize, usize);

pub struct DataAllocator<'long> {
    pubkey: Pubkey,
    data: &'long mut [u8],
    allocated_segments: BTreeMap<u32, SegmentOffset>,
    borrowed_serments: BTreeSet<u32>,
}

impl<'a> fmt::Debug for DataAllocator<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DataAllocator")
            .field("pubkey", &self.pubkey)
            .field("allocated_segments", &self.allocated_segments)
            .field("borrowed_serments", &self.borrowed_serments)
            .field("data_size", &self.data.len())
            .finish()
    }
}

impl<'a> DataAllocator<'a> {
    pub fn segment(&mut self, id: u32) -> Result<&'a mut [u8], DataError> {
        if self.borrowed_serments.contains(&id) {
            return Err(DataError::AlreadyBorrowed);
        }

        if let Some(&(offset_start, offset_end)) = self.allocated_segments.get(&id) {
            self.borrowed_serments.insert(id);

            unsafe {
                //TODO: document safety
                let data_ptr = self.data.as_mut_ptr();

                debug_assert!(offset_end <= self.data.len());
                debug_assert!(offset_start < offset_end);

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
        data: &'a mut [u8],
        allocated_segments: BTreeMap<u32, SegmentOffset>,
        borrowed_serments: BTreeSet<u32>,
    ) -> Self {
        Self {
            pubkey,
            data,
            allocated_segments,
            borrowed_serments,
        }
    }
}

pub enum DataError {
    NoSuchSegment,
    AlreadyBorrowed,
}
