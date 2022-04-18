use crate::db_manager::DBManager;
use crate::schemas_manager::{Schema, SchemaId, SchemasManager};
use borsh::{BorshDeserialize, BorshSerialize};
use slice_rbtree;
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

// TODO: specify by DB
type DBId = String;
type DBRequest = String;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    // TODO: add descriptions
    CreateDB { schema_id: SchemaId, db_id: DBId },
    RemoveDB { db_id: DBId },

    AddDBSchema { schema_id: SchemaId, schema: Schema },
    RemoveDBSchema { schema_id: SchemaId },
    UpdateDBSchema { schema_id: SchemaId, schema: Schema },

    CallDB { db_id: DBId, request: DBRequest },
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
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: Instruction,
) -> Result<(), Error> {
    let accounts_iter = &mut accounts.iter();
    match instruction {
        Instruction::CreateDB { schema_id, db_id } => {
            DBManager::create_db();
        }
        Instruction::RemoveDB { db_id } => {
            DBManager::remove_db();
        }
        Instruction::AddDBSchema { schema_id, schema } => {
            SchemasManager::add_schema();
        }
        Instruction::RemoveDBSchema { schema_id } => {
            SchemasManager::remove_schema();
        }
        Instruction::UpdateDBSchema { schema_id, schema } => {
            SchemasManager::update_schema();
        }
        Instruction::CallDB { db_id, request } => {
            DBManager::process_request();
        }
    }

    Ok(())
}
