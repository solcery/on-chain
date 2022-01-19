use super::memory::Error as MemoryError;
use borsh::{BorshDeserialize, BorshSerialize};
use thiserror::Error;

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
#[error("On instruction {instruction}: {source}")]
pub struct Error {
    pub instruction: u32,
    pub source: ErrorSource,
}

#[derive(Error, Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub enum ErrorSource {
    #[error("VM halted")]
    Halt,
    #[error("CardType index is out of bounds")]
    NoSuchType,
    #[error("Acsess violation: attempted to access memory region {region_index} not owned by {player_id}")]
    AccessViolation { player_id: u32, region_index: u32 },
    #[error("MemoryRegion index ({index}) is out of bounds")]
    NoSuchRegion { index: u32 },
    #[error("Memory Error")]
    MemoryError {
        #[from]
        source: MemoryError,
    },
}
