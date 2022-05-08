use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use std::cell::RefMut;
use std::collections::BTreeMap;

use super::{AccountAllocator, AllocatorError, DataAllocator, FSDispatcher, SegmentId};

#[derive(Debug)]
pub struct FSAllocator<'a> {
    allocators: BTreeMap<Pubkey, AccountAllocator<'a>>,
}

impl<'a> FSAllocator<'a> {
    /// Constructs [FSAllocator], assuming that all accounts are initialized as filesystem accounts
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

    /// Constructs [FSAllocator], assuming that some (or all) accounts may be uninitialized as filesystem accounts
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

    pub fn allocate_segment(&mut self, size: usize) -> Result<SegmentId, AllocatorError> {
        use AllocatorError::{NoInodesLeft, NoSuitableSegmentFound};

        let mut global_result = Err(NoSuitableSegmentFound);
        for (key, alloc) in self.allocators.iter_mut() {
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
        return global_result;
    }

    pub fn deallocate_segment(&mut self, id: SegmentId) -> Result<(), AllocatorError> {
        match self.allocators.get_mut(&id.pubkey) {
            Some(alloc) => alloc.deallocate_segment(id.id),
            None => Err(AllocatorError::NoSuchPubkey),
        }
    }

    pub fn to_fs_dispatcher(self) -> FSDispatcher<'a> {
        let allocators: BTreeMap<Pubkey, DataAllocator<'a>> = self
            .allocators
            .into_iter()
            .map(|(key, alloc)| (key, alloc.to_data_allocator()))
            .collect();

        unsafe { FSDispatcher::from_raw_parts(allocators) }
    }

    pub fn defragment_fs(&mut self) {
        for (_, alloc) in self.allocators.iter_mut() {
            alloc.merge_segments();
        }
    }
}
