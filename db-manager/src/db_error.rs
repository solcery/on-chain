use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum DBManagerError {
    /// Insufficient permissions to query to database.
    #[error("Query access denied")]
    QueryAccessDenied,
    /// Database with the specified id does not exist.
    #[error("Database doesn't exist.")]
    InvalidId,
}
impl From<DBManagerError> for ProgramError {
    fn from(e: DBManagerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for DBManagerError {
    fn type_of() -> &'static str {
        "DBManagerError"
    }
}
