use crate::{
    messages::{AddSchema, GetSchema, RemoveSchema, UpdateSchema},
    schemas_manager::SchemasManager,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    AddSchema { message: AddSchema },
    RemoveSchema { message: RemoveSchema },
    UpdateSchema { message: UpdateSchema },
    GetSchema { message: GetSchema },
}

entrypoint!(process_instruction_bytes);
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
    let mut _data = states_account_info.try_borrow_mut_data()?;

    match instruction {
        Instruction::AddSchema { message } => {
            SchemasManager::add_schema(message)?;
        }
        Instruction::RemoveSchema { message } => {
            SchemasManager::remove_schema(message)?;
        }
        Instruction::UpdateSchema { message } => {
            SchemasManager::update_schema(message)?;
        }
        Instruction::GetSchema { message } => {
            SchemasManager::get_schema(message)?;
        }
    }

    Ok(())
}
