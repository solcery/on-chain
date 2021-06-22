use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum SolceryError {
    #[error("Invalid instruction")]
    InvalidInstruction,
    #[error("Not a player! First join the board via JoinBoard instruction to be able to play.")]
    NotAPlayer,
    #[error("WrongCard")]
    WrongCard,
    #[error("InGameError")]
    InGameError,
    #[error("Game started")]
    GameStarted,
}

impl From<SolceryError> for ProgramError {
    fn from(e: SolceryError) -> Self {
        ProgramError::Custom(e as u32)
    }
}