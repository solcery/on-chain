use super::memory::Error as InternalError;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Copy, Clone, Debug, Eq, PartialEq, BorshDeserialize, BorshSerialize)]
pub struct Error {
    pub instruction: u32,
    pub error: InternalError,
}
