use crate::card::Card;
use crate::word::Word;
use borsh::{BorshDeserialize, BorshSerialize};
use thiserror::Error;

mod memory_region;
pub use memory_region::MemoryRegion;

mod player_board;
pub use player_board::PlayerBoard;

mod verification_board;
pub use verification_board::VerificationBoard;

pub trait Board {
    fn generate_card_id(&mut self) -> u32;
    fn memory_region(&mut self, region_index: usize) -> Result<&mut MemoryRegion, Error>;
    #[must_use]
    fn owner(&self) -> u32;
    fn iter_mut(&mut self) -> IterMut<'_, Self>
    where
        Self: Sized,
    {
        IterMut {
            counter: 0,
            board: self,
        }
    }
}

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    #[error("Access violation: player_id: {player_id} attempted to access not readable memory region {region_index}")]
    AccessViolation { player_id: u32, region_index: u32 },
    #[error("MemoryRegion index ({index}) is out of bounds")]
    NoSuchRegion { index: u32 },
}
pub struct IterMut<'a, Brd: Board> {
    counter: usize,
    board: &'a mut Brd,
}

impl<'a, Brd: Board> Iterator for IterMut<'a, Brd> {
    type Item = &'a mut MemoryRegion;

    fn next(&mut self) -> Option<Self::Item> {
        let counter = &mut self.counter;
        let board = &mut self.board;
        loop {
            match board.memory_region(*counter) {
                Ok(region) => {
                    *counter += 1;
                    let raw_ptr = region as *mut MemoryRegion;
                    // SAFETY: it is guaranted, that value of counter will be incremented each
                    // time, so this will newer create two references to one region.
                    return Some(unsafe { &mut *raw_ptr });
                }
                Err(Error::AccessViolation { .. }) => {
                    *counter += 1;
                    continue;
                }
                Err(Error::NoSuchRegion { .. }) => {
                    return None;
                }
            }
        }
    }
}
