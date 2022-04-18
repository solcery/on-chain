use crate::{
    db_manager::{DBId, DBManager, DBQuery},
    schemas_manager::{Schema, SchemaId, SchemasManager},
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    CreateDB {
        schema_id: SchemaId,
        db_id: DBId,
    },
    RemoveDB {
        db_id: DBId,
    },

    AddSchema {
        schema_id: SchemaId,
        schema: Schema,
    },
    RemoveSchema {
        schema_id: SchemaId,
    },
    UpdateSchema {
        schema_id: SchemaId,
        new_schema: Schema,
    },

    Query {
        db_id: DBId,
        query: DBQuery,
    },
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
    let _accounts_iter = &mut accounts.iter();

    match instruction {
        Instruction::CreateDB { schema_id, db_id } => {
            DBManager::create_db(schema_id, db_id)?;
        }
        Instruction::RemoveDB { db_id } => {
            DBManager::remove_db(db_id)?;
        }
        Instruction::AddSchema { schema_id, schema } => {
            SchemasManager::add_schema(schema_id, schema)?;
        }
        Instruction::RemoveSchema { schema_id } => {
            SchemasManager::remove_schema(schema_id)?;
        }
        Instruction::UpdateSchema {
            schema_id,
            new_schema,
        } => {
            SchemasManager::update_schema(schema_id, new_schema)?;
        }
        Instruction::Query { db_id, query } => {
            DBManager::process_query(db_id, query)?;
        }
    }

    Ok(())
}
