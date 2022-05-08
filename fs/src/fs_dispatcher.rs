use solana_program::pubkey::Pubkey;
use std::collections::BTreeMap;

use crate::{DataAllocator, DataError, SegmentId};

#[derive(Debug)]
pub struct FSDispatcher<'a> {
    allocators: BTreeMap<Pubkey, DataAllocator<'a>>,
}

impl<'long: 'short, 'short> FSDispatcher<'long> {
    pub(crate) unsafe fn from_raw_parts(
        allocators: BTreeMap<Pubkey, DataAllocator<'long>>,
    ) -> Self {
        Self { allocators }
    }

    pub fn segment(&mut self, id: SegmentId) -> Result<&'short mut [u8], DataError> {
        match self.allocators.get_mut(&id.pubkey) {
            Some(alloc) => alloc.segment(id.id),
            None => Err(DataError::NoSuchPubkey),
        }
    }
}
