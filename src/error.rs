use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum VMError {
    #[error("No enough data in instruction_data")]
    InstructionDataTooShort,
    #[error(
        "VM has reached the maximum number of instructions, but hasn't finished action evaluation"
    )]
    ComputationNotFinished,
}

impl From<VMError> for ProgramError {
    fn from(e: VMError) -> Self {
        Self::Custom(e as u32)
    }
}
