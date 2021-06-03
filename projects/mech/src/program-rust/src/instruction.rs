use solana_program::program_error::ProgramError;
use crate::error::BricksError;

pub enum GrimmzInstruction{

    CreateCard {
        data: Vec<u8>,
    },
    Execute
}

impl GrimmzInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(BricksError::InvalidInstruction)?; 
        Ok(match tag {
            0 => Self::Execute,
            1 => Self::CreateCard{ data: rest.to_vec() },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }
}
