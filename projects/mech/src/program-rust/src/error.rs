use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum BricksError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<BricksError> for ProgramError {
    fn from(e: BricksError) -> Self {
        ProgramError::Custom(e as u32)
    }
}