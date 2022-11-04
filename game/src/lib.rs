#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]
// FIXME: this crate needs heavy refactoring
// especially, state module
// I should reorganize the code according to solana's standard

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod entrypoint;
pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

mod game_state;
