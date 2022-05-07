use crate::db_manager::DBManager;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solcery_data_types::db::messages::db_manager::{CreateDB, Query, RemoveDB};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    Create { message: CreateDB },
    Remove { message: RemoveDB },

    Query { message: Query },
}

pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = Instruction::deserialize(&mut buf)?;

    // TODO: We probaly need a special feature for debug printing
    if cfg!(debug_assertions) {
        dbg!(process_instruction(program_id, accounts, instruction)).map_err(ProgramError::from)
    } else {
        process_instruction(program_id, accounts, instruction).map_err(ProgramError::from)
    }
}

fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: Instruction,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let states_account_info = next_account_info(account_info_iter)?;
    let data = states_account_info.try_borrow_mut_data()?;

    match instruction {
        Instruction::Create { message } => {
            DBManager::create_db(message, data)?;
        }
        Instruction::Remove { message } => {
            DBManager::remove_db(message, data)?;
        }
        Instruction::Query { message } => {
            DBManager::process_query(message, data)?;
        }
    }

    Ok(())
}
