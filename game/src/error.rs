use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Clone, Eq, PartialEq, Debug)]
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
    #[error("Account contain no data")]
    EmptyAccount,
    #[error("Account data corrupted")]
    CorruptedAccount,
    #[error("Wrong GameProject account")]
    WrongProjectAccount,
    #[error("ProgramError in the underlying code")]
    ProgramError(ProgramError),
    #[error("Player is already participating in another game")]
    AlreadyInGame,
    #[error("Attempted to join a game, which is already started")]
    GameStarted,
    #[error("No player slots left")]
    NoPlayerSlots,
    #[error("Player not in this game")]
    NotInGame,
    #[error("The game is not finished")]
    NotFinished,
    #[error("Account is not owned by Game program")]
    WrongAccountOwner,
}

impl From<Error> for ProgramError {
    fn from(error: Error) -> Self {
        match error {
            Error::ProgramError(e) => e,
            //FIXME: create proper conversion from Error to ProgramError
            e => ProgramError::Custom(0),
        }
    }
}

impl From<ProgramError> for Error {
    fn from(error: ProgramError) -> Self {
        Error::ProgramError(error)
    }
}
