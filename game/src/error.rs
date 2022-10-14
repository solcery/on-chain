use super::state::game::Error as GameError;
use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    #[error("Player account for this signer is already created")]
    AlreadyCreated,
    #[error("Player is already participating in another game")]
    AlreadyInGame,
    #[error("Attempted to use already initialized account")]
    AlreadyInUse,
    #[error("Account data corrupted")]
    CorruptedAccount,
    #[error("Account contain no data")]
    EmptyAccount,
    #[error("The supplied token is not an NFT")]
    NotAnNFT,
    #[error("Player not in this game")]
    NotInGame,
    #[error("NFT is not owned by player")]
    NotOwnedNFT,
    #[error("Transaction is not signed")]
    NotSigned,
    #[error("ProgramError in the underlying code: {0}")]
    ProgramError(ProgramError),
    #[error("Game and State accounts does not match")]
    StateAccountMismatch,
    #[error("Mint in token account and key of the mint account does not match")]
    WrongAccountMint,
    #[error("Account is not owned by Game program")]
    WrongAccountOwner,
    #[error("You are using old version of Player account. Please, update it via UpdatePlayerAccount instruction.")]
    WrongAccountVersion,
    #[error("You are using old Game Project account.")]
    WrongProjectVersion,
    #[error("Address of the provided player account does not match the required PDA")]
    WrongPlayerAccount,
    #[error("This game can not be played with this number of players")]
    WrongPlayerNumber,
    #[error("Wrong GameProject account")]
    WrongProjectAccount,
    #[error("Wrong state step, you are probably out of sync")]
    WrongStateStep,
    #[error("GameError")]
    Game(GameError),
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

impl From<GameError> for Error {
    fn from(error: GameError) -> Self {
        Error::Game(error)
    }
}
