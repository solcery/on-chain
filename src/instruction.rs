use solana_program::{
    program_error::ProgramError,
    msg,
};
use crate::error::BricksError;


pub enum GrimmzInstruction{

    CreateCard {
        data: Vec<u8>,
    },
    CreateFight,
    Cast {
        caster_id: u8,
        target_id: u8,
    },
}

impl GrimmzInstruction {
    /// Unpacks a byte buffer into a [EscrowInstruction](enum.EscrowInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(BricksError::InvalidInstruction)?; 
        msg!("GrimmzInstruction");
        msg!("{:?}", tag);
        Ok(match tag {
            0 => Self::CreateCard{ data: rest.to_vec() },
            1 => Self::CreateFight,
            2 => Self::Cast{ caster_id: input[0], target_id: input[1] },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }
}
