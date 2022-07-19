//! Solcery DB program
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(missing_debug_implementations)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    pubkey::Pubkey,
};
use spl_token::id;
use std::cell::RefCell;
use std::rc::Rc;

use account_fs::{SegmentId, FS};
use solcery_db::{ColumnId, ColumnParams, Data, DataType, Error as DBError, DB};

pub const MINT_SEED: &[u8; 4] = b"mint";
pub const MINT_AUTHORITY_SEED: &[u8; 14] = b"mint_authority";

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

    if instruction == DBInstruction::Bootstrap {
        let admin = next_account_info(&mut account_iter)?;
        let mint = next_account_info(&mut account_iter)?;
        let mint_authority = next_account_info(&mut account_iter)?;
        let token_program = next_account_info(&mut account_iter)?;
        let token_account = next_account_info(&mut account_iter)?;
        let rent_sysvar = next_account_info(&mut account_iter)?;

        if !admin.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if token_program.key != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if !token_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let mint_id = Pubkey::create_program_address(&[MINT_SEED], &program_id)?;
        let mint_authority_id =
            Pubkey::create_program_address(&[MINT_AUTHORITY_SEED], &program_id)?;

        if mint.key != &mint_id {
            return Err(ProgramError::InvalidArgument);
        }
        if mint_authority.key != &mint_authority_id {
            return Err(ProgramError::InvalidArgument);
        }

        eprintln!("1");
        let init_mint_instruction = spl_token::instruction::initialize_mint(
            token_program.key,
            &mint.key,
            &mint_authority.key,
            None,
            10,
        )?;

        invoke(&init_mint_instruction, &[mint.clone(), rent_sysvar.clone()])?;
        eprintln!("2");

        let init_token_instruction = spl_token::instruction::initialize_account(
            token_program.key,
            &token_account.key,
            &mint.key,
            &admin.key,
        )?;

        invoke(
            &init_token_instruction,
            &[
                token_account.clone(),
                mint.clone(),
                admin.clone(),
                rent_sysvar.clone(),
            ],
        )?;
        eprintln!("3");

        let mint_token_instruction = spl_token::instruction::mint_to(
            token_program.key,
            &mint.key,
            &token_account.key,
            &admin.key,
            &[mint_authority.key],
            1,
        )?;

        invoke_signed(
            &mint_token_instruction,
            &[mint.clone(), token_account.clone(), mint_authority.clone()],
            &[&[MINT_AUTHORITY_SEED]],
        )?;
        eprintln!("4");
        todo!();
    } else {
        let token_account = next_account_info(&mut account_iter)?;

        if token_account.owner != &spl_token::id() {
            return Err(ProgramError::IncorrectProgramId);
        }

        if !token_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        if cfg!(debug_assertions) {
            dbg!(process_instruction(program_id, account_iter, instruction))
                .map_err(ProgramError::from)
        } else {
            process_instruction(program_id, account_iter, instruction).map_err(ProgramError::from)
        }
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
    /// Bootstrap Solcery DB-program
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will initiate DB.
    /// 1. `[writable]` Mint account
    /// 2. `[]` Mint authority account
    /// 3. `[]` Token Program
    /// 4. `[signer,writable]` Access Token account
    /// 5. `[]` Rent SysVar
    //let admin = next_account_info(&mut account_iter)?;
    //let mint = next_account_info(&mut account_iter)?;
    //let mint_authority = next_account_info(&mut account_iter)?;
    //let token_program = next_account_info(&mut account_iter)?;
    //let token_account = next_account_info(&mut account_iter)?;
    //let rent_sysvar = next_account_info(&mut account_iter)?;
    Bootstrap,
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
