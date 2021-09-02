use solana_program::{
    program_error::ProgramError,
};
use crate::error::SolceryError;
use borsh::BorshDeserialize;

use std::convert::TryInto;
use crate::fight_log::{
    FightLog,
};

#[derive(BorshDeserialize)]
pub struct HostBoardParams {
    pub send_stat: bool,
    pub public: bool,
    pub random_seed: u32,
}

#[derive(BorshDeserialize)]
pub struct JoinBoardParams {
    pub remove_from_lobby: bool,
    pub bot: bool,
}

pub enum SolceryInstruction{
    /// Checks and stores card binary data into special account\
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person creating the card
    /// 1. `[writable]` The account for card metadata storage, with allocated memory and owned by program
    /// 2. `[]` Mint account of card NFT

    SetEntity {
        position: u32,
        data: Vec<u8>,
    },

    /// Removes all lamports from card account to user account allowing to create new account with such key
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person creating the card
    /// 1. `[writable]` The account for card metadata storage, with allocated memory and owned by program
    DeleteEntity,

    /// Initializes new board and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the board
    /// 1. `[writable]` Memory account owned by program with preallocated necessary space
    /// 2+. [] Metadata account of cards used in board
    CompileBoard,

    /// Initializes new board and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person starting the board
    /// 1. `[writable]` Memory account owned by program with preallocated necessary space
    /// 2+. [] Metadata account of cards used in board
    AddCardsToBoard {
        cards_amount: u32,
    },

    /// Initializes new board and stores it in account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person joining the board
    /// 1. `[writable]` Active board account
    JoinBoard {
        params: JoinBoardParams,
    },

    /// Checks and stores card binary data into special account
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person creating the unit
    /// 1. `[writable]` The account for unit metadata storage, with allocated memory and owned by program
    /// 2. `[]` Mint account of card NFT
    AddLog {
        log: FightLog,
    },


    HostBoard {
        params: HostBoardParams,
    }

}

impl SolceryInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, mut rest) = input.split_first().ok_or(SolceryError::InvalidInstruction)?; 
        Ok(match tag {
            0 => {
                let (position_slice, data) = rest.split_at(4);
                Self::SetEntity { 
                    position: u32::from_le_bytes(position_slice.try_into().unwrap()),
                    data: data.to_vec() ,
                }
            }
            1 => Self::DeleteEntity,
            2 => Self::CompileBoard,
            3 => Self::AddCardsToBoard { cards_amount: u32::from_le_bytes(rest.try_into().unwrap()) },
            4 => Self::JoinBoard { params: JoinBoardParams::deserialize(&mut rest)? },
            5 => Self::AddLog { log: FightLog::deserialize(&mut rest)?  },
            6 => Self::HostBoard { params: HostBoardParams::deserialize(&mut rest)? },
            _ => return Err(ProgramError::InvalidAccountData.into()),
        })
    }
}
