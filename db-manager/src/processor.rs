use borsh::{BorshDeserialize, BorshSerialize};
use slice_rbtree;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
struct Schema {
    version: u64,
    tables: String, //TODO: how store tables order/types?
}

type SchemaId = String;
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
            unimplemented!();
        }
        Instruction::RemoveDB { db_id } => {
            unimplemented!();
        }
        Instruction::AddDBSchema { schema_id, schema } => {
            unimplemented!();
        }
        Instruction::RemoveDBSchema { schema_id } => {
            unimplemented!();
        }
        Instruction::UpdateDBSchema { schema_id, schema } => {
            unimplemented!();
        }
        Instruction::CallDB { db_id, request } => {
            unimplemented!();
        }
    }

    Ok(())
}
