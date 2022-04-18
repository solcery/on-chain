use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum SchemasManagerError {
    /// The schema with the specified id does not exist.
    #[error("There is no schema with this id")]
    IncorrectId,
    /// The column type is not supported.
    #[error("Unsupported column type")]
    UnsupportedType,
}
impl From<SchemasManagerError> for ProgramError {
    fn from(e: SchemasManagerError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for SchemasManagerError {
    fn type_of() -> &'static str {
        "SchemasManagerError"
    }
}
