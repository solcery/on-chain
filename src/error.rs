use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum VMError {
    #[error("No enough data in instruction_data")]
    InstructionDataTooShort,
}

impl From<VMError> for ProgramError {
    fn from(e: VMError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
