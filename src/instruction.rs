use solana_program::program_error::ProgramError;
use crate::error::SolceryError;


pub enum SolceryInstruction{

    /// Checks and stores card binary data into special account
    /// Accounts expected:
    ///
    /// 2  `[signer]` The account of the person creating the card
    /// 0. `[writable]` The account for card metadata storage, with allocated memory and owned by program
    /// 1. `[]` Mint account of card NFT

    CreateCard {
        data: Vec<u8>,
    },

    /// Initializes new fight and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the fight
    /// 1. `[writable]` Memory account owned by program with preallocated necessary space
    CreateFight,

    /// Gives 
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person casting the card
    /// 1. `[writable]` Fight account
    /// 2. `[]` Card metadata account
    Cast {
        caster_id: u8, // [ignored, always 1] // Id of unit which will cast the card
        target_id: u8, // [ignored, always 2] // Id of unit which will be the target of the card
    },
}

impl SolceryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(SolceryError::InvalidInstruction)?; 
        Ok(match tag {
            0 => Self::CreateCard{ data: rest.to_vec() },
            1 => Self::CreateFight,
            2 => Self::Cast{ caster_id: input[0], target_id: input[1] },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }
}
