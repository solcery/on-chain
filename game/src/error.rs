use solana_program::program_error::ProgramError;
use thiserror::Error;

#[derive(Error, Clone, Eq, PartialEq, Debug)]
pub enum Error {
    #[error("Player account for this signer is already created")]
    AlreadyCreated,
    #[error("Player is already participating in another game")]
    AlreadyInGame,
    #[error("Account data corrupted")]
    CorruptedAccount,
    #[error("Account contain no data")]
    EmptyAccount,
    #[error("The game has already started")]
    GameStarted,
    #[error("Illegal status change")]
    IllegalStatusChange,
    #[error("No player slots left")]
    NoPlayerSlots,
    #[error("Not all players have joined the game")]
    NotAllPlayersReady,
    #[error("The supplied token is not an NFT")]
    NotAnNFT,
    #[error("The game is not finished")]
    NotFinished,
    #[error("Player not in this game")]
    NotInGame,
    #[error("NFT is not owned by player")]
    NotOwnedNFT,
    #[error("Transaction is not signed")]
    NotSigned,
    #[error("ProgramError in the underlying code: {0}")]
    ProgramError(ProgramError),
    #[error("The supplied token is already in game")]
    TokenAlreadyInGame,
    #[error("Attempted to add too many items")]
    TooManyItems,
    #[error("Mint in token account and key of the mint account does not match")]
    WrongAccountMint,
    #[error("Account is not owned by Game program")]
    WrongAccountOwner,
    #[error("You are using old version of Player account. Please, update it via UpdatePlayerAccount instruction.")]
    WrongAccountVersion,
    #[error("Address of the provided player account does not match the required PDA")]
    WrongPlayerAccount,
    #[error("This game can not be played with this number of players")]
    WrongPlayerNumber,
    #[error("Wrong GameProject account")]
    WrongProjectAccount,
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
