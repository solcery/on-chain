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

const INODE_TABLE_SIZE: usize = 100;

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
        SetValueSecondary(params) => process_set_value_secondary(program_id, accounts, params),
        _ => unimplemented!(),
    }
}

fn process_set_value(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SetValueParams,
) -> Result<(), DBError> {
    let mut db = prepare_db(program_id, accounts, params.db, params.is_initialized)?;
    db.set_value(params.key, params.column, params.value)
        .map(|_| ())
}

fn process_set_value_secondary(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    params: SetValueSecondaryParams,
) -> Result<(), DBError> {
    let mut db = prepare_db(program_id, accounts, params.db, params.is_initialized)?;
    db.set_value_secondary(
        params.key_column,
        params.secondary_key,
        params.value_column,
        params.value,
    )
    .map(|_| ())
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub enum DBInstruction {
    SetValue(SetValueParams),
    SetValueSecondary(SetValueSecondaryParams),
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
    /// Are all the FS accounts initialized
    is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetValueSecondaryParams {
    db: SegmentId,
    key_column: ColumnId,
    secondary_key: Data,
    value_column: ColumnId,
    value: Data,
    /// Are all the FS accounts initialized
    is_initialized: bool,
}

fn prepare_db<'a>(
    program_id: &Pubkey,
    accounts: &'a [AccountInfo],
    segment: SegmentId,
    is_initialized: bool,
) -> Result<DB<'a>, DBError> {
    let account_iter = &mut accounts.iter();

    let fs = if is_initialized {
        FS::from_account_iter(program_id, account_iter)?
    } else {
        FS::from_uninit_account_iter(program_id, account_iter, INODE_TABLE_SIZE)?
    };

    let fs_cell = Rc::new(RefCell::new(fs));

    DB::from_segment(fs_cell, segment)
}
