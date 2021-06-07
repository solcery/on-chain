use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum SolceryError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<SolceryError> for ProgramError {
    fn from(e: SolceryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}