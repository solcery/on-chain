use solana_program::{
    program_error::ProgramError,
};
use crate::error::SolceryError;
use crate::board::Place;
use borsh::BorshDeserialize;
use std::convert::TryInto;


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

    /// Initializes new board and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the board
    /// 1. `[writable]` Memory account owned by program with preallocated necessary space
    /// 2+. [] Metadata account of cards used in board
    CreateBoard {
        deck: Vec<(u32, Place)>,
        init: Vec<u32>,
    },

    /// Initializes new board and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person joining the board
    /// 1. `[writable]` Active board account
    JoinBoard,

    /// Checks and stores card binary data into special account
    /// Accounts expected:
    ///
    /// 0  `[signer]` The account of the person creating the unit
    /// 1. `[writable]` The account for unit metadata storage, with allocated memory and owned by program
    /// 2. `[]` Mint account of card NFT
    Cast {
        card_id: u32,
    },



}

impl SolceryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (mut tag, mut rest) = input.split_first().ok_or(SolceryError::InvalidInstruction)?; 
        Ok(match tag {
            0 => Self::CreateCard{ data: rest.to_vec() },
            1 => Self::CreateBoard{ 
                deck: Vec::<(u32, Place)>::deserialize(&mut rest)?,
                init: Vec::<u32>::deserialize(&mut rest)?,
            },
            2 => Self::JoinBoard,
            3 => Self::Cast{ 
                card_id: u32::from_le_bytes(rest[..4].try_into().unwrap()),
            },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }

    pub fn unpack_board_deck(src: &[u8]) -> Vec<(u32, Place)> {
        let mut deck = Vec::new();
        for i in 0..src.len() / 5 {
            let amount = u32::from_le_bytes(src[5 * i .. 5 * (i + 1) - 1].try_into().unwrap());
            let place_u8 = src[5 * (i + 1) - 1];
            deck.push((amount, Place::from_u8(place_u8)));
        }
        return deck
    }

}
