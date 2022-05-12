use crate::{db_manager::DBManager, schemas_manager::SchemasManager};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solcery_data_types::db::messages::db_manager::{CreateDB, Query, RemoveDB};
use solcery_data_types::db::messages::schemas_manager::{
    AddSchema, GetSchema, RemoveSchema, UpdateSchema,
};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum DataBaseInstruction {
    AddSchema { message: AddSchema },
    RemoveSchema { message: RemoveSchema },
    UpdateSchema { message: UpdateSchema },
    GetSchema { message: GetSchema },

    Create { message: CreateDB },
    Remove { message: RemoveDB },

    Query { message: Query },
}

entrypoint!(process_instruction_bytes);
pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = DataBaseInstruction::deserialize(&mut buf)?;

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
    instruction: DataBaseInstruction,
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();
    let states_account_info = next_account_info(account_info_iter)?;
    let data = states_account_info.try_borrow_mut_data()?;

    match instruction {
        DataBaseInstruction::AddSchema { message } => {
            SchemasManager::add_schema(message, data)?;
        }
        DataBaseInstruction::RemoveSchema { message } => {
            SchemasManager::remove_schema(message, data)?;
        }
        DataBaseInstruction::UpdateSchema { message } => {
            SchemasManager::update_schema(message, data)?;
        }
        DataBaseInstruction::GetSchema { message } => {
            let states_account_info = next_account_info(account_info_iter)?;
            let res_data = states_account_info.try_borrow_mut_data()?;

            SchemasManager::get_schema(message, data, res_data)?;
        }
        DataBaseInstruction::Create { message } => {
            DBManager::create_db(message, data)?;
        }
        DataBaseInstruction::Remove { message } => {
            DBManager::remove_db(message, data)?;
        }
        DataBaseInstruction::Query { message } => {
            let states_account_info = next_account_info(account_info_iter)?;
            let res_data = states_account_info.try_borrow_mut_data()?;

            DBManager::process_query(message, data, res_data)?;
        }
    }

    Ok(())
}
