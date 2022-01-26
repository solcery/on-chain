use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(
    Error, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize,
)]
pub enum Error {
    #[error("Player account for this signer is already created")]
    AlreadyCreated,
    #[error("Player account not writable")]
    NotWritable,
    #[error("Player account too small")]
    // This error should be converted to AccountDataTooSmall
    AccountTooSmall,
    #[error("Address of the provided player account does not match the required PDA")]
    WrongPlayerAccount,
    #[error("You are using old version of Player account. Please, update it via UpdatePlayerAccount instruction.")]
    WrongAccountVersion,
    #[error("Transaction is not signed")]
    NotSigned,
}

impl From<Error> for ProgramError {
    fn from(error: Error) -> Self {
        ProgramError::Custom(error as u32)
    }
}
