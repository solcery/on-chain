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
    // TODO: Error struct
    fn generate_card_id(&mut self) -> u32;
    fn memory_region(
        &mut self,
        region_index: usize,
        player_id: u32,
    ) -> Result<&mut MemoryRegion, Error>;
    #[must_use]
    fn region_count(&self) -> usize;
}

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum Error {
    #[error("Access violation: player_id: {player_id} attempted to access not readable memory region {region_index}")]
    AccessViolation { player_id: u32, region_index: u32 },
    #[error("MemoryRegion index ({index}) is out of bounds")]
    NoSuchRegion { index: u32 },
}
