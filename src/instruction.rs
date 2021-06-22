use solana_program::{
    program_error::ProgramError,
    msg
};
use crate::board::PlaceId;
use crate::error::SolceryError;


pub enum SolceryInstruction{

    /// Checks and stores card binary data into special account
    /// Accounts expected:
    ///
    /// 0  `[signer]` The account of the person creating the card
    /// 1. `[writable]` The account for card metadata storage, with allocated memory and owned by program
    /// 2. `[]` Mint account of card NFT

    CreateCard {
        data: Vec<u8>,
    },
    /// Initializes new fight and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the fight
    /// 1. `[writable]` Memory account owned by program with preallocated necessary space
    CreateFight,

    /// Casts spell from unit to target position
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person casting the card
    /// 1. `[writable]` Fight account
    /// 2. `[]` Card metadata account
    Cast {
        caster_id: u32,
        position: PlaceId,
    },

    /// Adds unit to battlefield
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person creating the unit
    /// 1. `[writable]` Fight account
    /// 2. `[]` Unit metadata account
    SpawnUnit {
        position: PlaceId
    },

    /// Checks and stores card binary data into special account
    /// Accounts expected:
    ///
    /// 0  `[signer]` The account of the person creating the unit
    /// 1. `[writable]` The account for unit metadata storage, with allocated memory and owned by program
    /// 2. `[]` Mint account of unit NFT
    CreateUnit {
        data: Vec<u8>,
    },



}

impl SolceryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(SolceryError::InvalidInstruction)?; 
        Ok(match tag {
            0 => Self::CreateCard{ data: rest.to_vec() },
            1 => Self::CreateFight,
            2 => Self::Cast{ 
                caster_id: u32::from_le_bytes(rest[..4].try_into().unwrap()),
                position: (u32::from_le_bytes(rest[4..8].try_into().unwrap()), u32::from_le_bytes(rest[8..12].try_into().unwrap())), 
            },
            3 => Self::SpawnUnit { 
                position: (u32::from_le_bytes(rest[..4].try_into().unwrap()), u32::from_le_bytes(rest[4..8].try_into().unwrap())),
            },
            4 => Self::CreateUnit{ data: rest.to_vec() },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }
}
