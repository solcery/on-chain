//! Solcery DB program
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use account_fs::{SegmentId, FS};
use solcery_db::{ColumnId, ColumnParams, Data, DataType, Error as DBError, DB};

entrypoint!(process_instruction_bytes);
pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = DBInstruction::deserialize(&mut buf)?;

    if cfg!(debug_assertions) {
        dbg!(process_instruction(program_id, accounts, instruction)).map_err(ProgramError::from)
    } else {
        process_instruction(program_id, accounts, instruction).map_err(ProgramError::from)
    }
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: DBInstruction,
) -> Result<(), DBError> {
    use DBInstruction::*;
    match instruction {
        SetValue(params) => process_set_value(program_id, accounts, params),
        _ => unimplemented!(),
    }
}

fn process_set_value(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SetValueParams,
) -> Result<(), DBError> {
    let mut db = prepare_db(accounts, params.db)?;
    let result = db.set_value(params.key, params.column, params.value);
    match result {
        Ok(_) => Ok(()),
        Err(err) => Err(DBError::from(err)),
    }
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub enum DBInstruction {
    SetValue(SetValueParams),
    SetValueSecondary {
        db: SegmentId,
        key_column: ColumnId,
        secondary_key: Data,
        value_column: ColumnId,
        value: Data,
    },
    SetRow {
        db: SegmentId,
        key: Data,
        row: BTreeMap<ColumnId, Data>,
    },
    DeleteRow {
        db: SegmentId,
        key: Data,
    },
    DeleteRowSecondary {
        db: SegmentId,
        secondary_key: Data,
        key_column: ColumnId,
    },
    CreateDB {
        primary_key_type: DataType,
        columns: Vec<ColumnParams>,
    },
    RemoveDB {
        db: SegmentId,
    },
    MintNewAccessToken,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetValueParams {
    db: SegmentId,
    column: ColumnId,
    key: Data,
    value: Data,
}

fn prepare_db<'a>(accounts: &'a [AccountInfo], segment: SegmentId) -> Result<DB<'a>, DBError> {
    let account_iter = &mut accounts.iter();
    let fs = FS::from_account_iter(account_iter)?;
    let fs_cell = Rc::new(RefCell::new(fs));

    DB::from_segment(fs_cell, segment)
}
