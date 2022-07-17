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

    let mut account_iter = &mut accounts.iter();
    if cfg!(debug_assertions) {
        dbg!(process_instruction(program_id, account_iter, instruction)).map_err(ProgramError::from)
    } else {
        process_instruction(program_id, account_iter, instruction).map_err(ProgramError::from)
    }
}

fn process_instruction<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
    instruction: DBInstruction,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    use DBInstruction::*;
    match instruction {
        SetValue(params) => process_set_value(program_id, account_iter, params),
        SetValueSecondary(params) => process_set_value_secondary(program_id, account_iter, params),
        SetRow(params) => process_set_row(program_id, account_iter, params),
        DeleteRow(params) => process_delete_row(program_id, account_iter, params),
        DeleteRowSecondary(params) => {
            process_delete_row_secondary(program_id, account_iter, params)
        }
        _ => unimplemented!(),
    }
}

fn process_set_value<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
    params: SetValueParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, account_iter, params.db, params.is_initialized)?;
    db.set_value(params.key, params.column, params.value)
        .map(|_| ())
}

fn process_set_value_secondary<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: SetValueSecondaryParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.set_value_secondary(
        params.key_column,
        params.secondary_key,
        params.value_column,
        params.value,
    )
    .map(|_| ())
}

fn process_set_row<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: SetRowParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.set_row(params.key, params.row).map(|_| ())
}

fn process_delete_row<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: DeleteRowParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.delete_row(params.key)
}
fn process_delete_row_secondary<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: DeleteRowSecondaryParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.delete_row_secondary(params.key_column, params.secondary_key)
        .map(|_| ())
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub enum DBInstruction {
    SetValue(SetValueParams),
    SetValueSecondary(SetValueSecondaryParams),
    SetRow(SetRowParams),
    DeleteRow(DeleteRowParams),
    DeleteRowSecondary(DeleteRowSecondaryParams),
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

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct SetRowParams {
    db: SegmentId,
    key: Data,
    row: Vec<(ColumnId, Data)>,
    /// Are all the FS accounts initialized
    is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct DeleteRowParams {
    db: SegmentId,
    key: Data,
    /// Are all the FS accounts initialized
    is_initialized: bool,
}

#[derive(BorshDeserialize, BorshSerialize, Clone, Debug, Eq, PartialEq)]
pub struct DeleteRowSecondaryParams {
    db: SegmentId,
    secondary_key: Data,
    key_column: ColumnId,
    /// Are all the FS accounts initialized
    is_initialized: bool,
}

fn prepare_db<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
    segment: SegmentId,
    is_initialized: bool,
) -> Result<DB<'short>, DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let fs = if is_initialized {
        FS::from_account_iter(program_id, account_iter)?
    } else {
        FS::from_uninit_account_iter(program_id, account_iter, INODE_TABLE_SIZE)?
    };

    let fs_cell = Rc::new(RefCell::new(fs));

    DB::from_segment(fs_cell, segment)
}
