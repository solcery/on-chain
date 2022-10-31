use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    system_instruction, system_program,
};
use spl_token::{
    instruction as token_instruction, state::Account as TokenAccount, state::Mint, ID as TokenID,
};
use std::cell::RefCell;
use std::rc::Rc;

use account_fs::{SegmentId, FS};
use solcery_db::{Error as DBError, DB};

use super::instruction::*;

pub use super::state::{DBGlobalState, GLOBAL_STATE_SEED, MINT_SEED};

const INODE_TABLE_SIZE: usize = 100;

pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = DBInstruction::deserialize(&mut buf)?;

    let account_iter = &mut accounts.iter();

    match instruction {
        DBInstruction::Bootstrap(params) => bootstrap(program_id, account_iter, params),
        DBInstruction::MintNewAccessToken => mint_new_token(program_id, account_iter),
        other_instruction => {
            check_token(program_id, account_iter)?;

            if cfg!(debug_assertions) {
                dbg!(process_instruction(
                    program_id,
                    account_iter,
                    other_instruction
                ))
                .map_err(ProgramError::from)
            } else {
                process_instruction(program_id, account_iter, other_instruction)
                    .map_err(ProgramError::from)
            }
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
        CreateDB(params) => process_create_db(program_id, account_iter, params),
        DropDB(segment) => process_drop_db(program_id, account_iter, segment),
        Bootstrap(_) => unreachable!("Bootstrap instruction should be handled separately"),
        MintNewAccessToken => {
            unreachable!("MintNewAccessToken instruction should be handled separately")
        }
        AddColumn(params) => process_add_column(program_id, account_iter, params),
        RemoveColumn(params) => process_remove_column(program_id, account_iter, params),
    }
}

fn bootstrap<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
    params: BootstrapParams,
) -> ProgramResult
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let admin = next_account_info(account_iter)?;
    let mint = next_account_info(account_iter)?;
    let global_state = next_account_info(account_iter)?;
    let token_account = next_account_info(account_iter)?;
    let system_program = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let rent_sysvar = next_account_info(account_iter)?;

    if !admin.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !system_program::check_id(system_program.key) {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !token_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // here we calculate bumps, so it is impossible to call Bootstrap twice with different PDAs
    let (mint_id, _mint_bump) = Pubkey::find_program_address(&[MINT_SEED], program_id);
    let (global_state_id, _state_bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], program_id);

    if mint.key != &mint_id {
        return Err(ProgramError::InvalidArgument);
    }
    if global_state.key != &global_state_id {
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Creating GlobalState account");
    let create_state_instruction = system_instruction::create_account(
        admin.key,
        global_state.key,
        params.lamports_to_global_state,
        DBGlobalState::get_packed_len() as u64,
        program_id,
    );

    invoke_signed(
        &create_state_instruction,
        &[admin.clone(), global_state.clone()],
        &[&[GLOBAL_STATE_SEED, &[params.state_bump]]],
    )?;

    msg!("Creating mint account");
    let create_mint_instruction = system_instruction::create_account(
        admin.key,
        mint.key,
        params.lamports_to_mint,
        Mint::get_packed_len() as u64,
        &TokenID,
    );

    invoke_signed(
        &create_mint_instruction,
        &[admin.clone(), mint.clone()],
        &[&[MINT_SEED, &[params.mint_bump]]],
    )?;

    msg!("Initializing mint");
    let init_mint_instruction =
        token_instruction::initialize_mint(token_program.key, mint.key, global_state.key, None, 0)?;

    invoke(&init_mint_instruction, &[mint.clone(), rent_sysvar.clone()])?;

    msg!("Initializing token");
    let init_token_instruction = token_instruction::initialize_account(
        token_program.key,
        token_account.key,
        mint.key,
        admin.key,
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

    msg!("Minting token");
    let mint_token_instruction = token_instruction::mint_to(
        token_program.key,
        mint.key,
        token_account.key,
        global_state.key,
        &[],
        1,
    )?;

    invoke_signed(
        &mint_token_instruction,
        &[mint.clone(), token_account.clone(), global_state.clone()],
        &[&[GLOBAL_STATE_SEED, &[params.state_bump]]],
    )?;

    let global_state_data = DBGlobalState::new(params.state_bump, params.mint_bump);
    DBGlobalState::pack(global_state_data, &mut global_state.data.borrow_mut())?;

    Ok(())
}

fn mint_new_token<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
) -> ProgramResult
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let admin = next_account_info(account_iter)?;
    let mint = next_account_info(account_iter)?;
    let global_state = next_account_info(account_iter)?;
    let token_account = next_account_info(account_iter)?;
    let new_token_account = next_account_info(account_iter)?;
    let token_program = next_account_info(account_iter)?;
    let rent_sysvar = next_account_info(account_iter)?;

    if token_program.key != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !token_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let global_state_data = DBGlobalState::unpack(&global_state.data.borrow())?;

    // here we calculate bumps, so it is impossible to call Bootstrap twice with different PDAs
    let mint_id =
        Pubkey::create_program_address(&[MINT_SEED, &[global_state_data.mint_bump()]], program_id)?;
    let global_state_id = Pubkey::create_program_address(
        &[GLOBAL_STATE_SEED, &[global_state_data.global_state_bump()]],
        program_id,
    )?;

    if mint.key != &mint_id {
        return Err(ProgramError::InvalidArgument);
    }
    if global_state.key != &global_state_id {
        return Err(ProgramError::InvalidArgument);
    }

    msg!("Initializing token");
    let init_token_instruction = token_instruction::initialize_account(
        token_program.key,
        new_token_account.key,
        mint.key,
        admin.key,
    )?;

    invoke(
        &init_token_instruction,
        &[
            new_token_account.clone(),
            mint.clone(),
            admin.clone(),
            rent_sysvar.clone(),
        ],
    )?;

    msg!("Minting token");
    let mint_token_instruction = token_instruction::mint_to(
        token_program.key,
        mint.key,
        new_token_account.key,
        global_state.key,
        &[],
        1,
    )?;

    invoke_signed(
        &mint_token_instruction,
        &[
            mint.clone(),
            new_token_account.clone(),
            global_state.clone(),
        ],
        &[&[GLOBAL_STATE_SEED, &[global_state_data.global_state_bump()]]],
    )?;

    Ok(())
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

fn process_create_db<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: CreateDBParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let fs = if params.is_initialized {
        FS::from_account_iter(program_id, accounts_iter)?
    } else {
        FS::from_uninit_account_iter(program_id, accounts_iter, INODE_TABLE_SIZE)?
    };

    let fs_cell = Rc::new(RefCell::new(fs));

    DB::init_in_segment(
        fs_cell,
        &params.table_name,
        params.max_columns as usize,
        params.max_rows as usize,
        params.primary_key_type,
    )
    .map(|_| ())
}
fn process_drop_db<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    segment: SegmentId,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let db = prepare_db(program_id, accounts_iter, segment, true)?;
    db.drop_db()
}

fn process_add_column<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: AddColumnParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.add_column(&params.name, params.dtype, params.is_secondary_key)
        .map(|_| ())
}

fn process_remove_column<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    accounts_iter: &mut AccountIter,
    params: RemoveColumnParams,
) -> Result<(), DBError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    let mut db = prepare_db(program_id, accounts_iter, params.db, params.is_initialized)?;
    db.remove_column(params.column_id).map(|_| ())
}

fn prepare_db<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
    segment: SegmentId,
    is_initialized: bool,
) -> Result<DB<'long, 'short>, DBError>
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

fn check_token<'long: 'short, 'short, AccountIter>(
    program_id: &Pubkey,
    account_iter: &mut AccountIter,
) -> Result<(), ProgramError>
where
    AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
{
    // GlobalState check
    let global_state_account = next_account_info(account_iter)?;

    let global_state = DBGlobalState::unpack(&global_state_account.data.borrow())?;

    let global_state_address = Pubkey::create_program_address(
        &[GLOBAL_STATE_SEED, &[global_state.global_state_bump()]],
        program_id,
    )?;

    if global_state_account.key != &global_state_address {
        return Err(ProgramError::InvalidArgument);
    }

    //Token check
    let token_account = next_account_info(account_iter)?;

    let token = TokenAccount::unpack(&token_account.data.borrow())?;

    if token_account.owner != &spl_token::id() {
        return Err(ProgramError::IncorrectProgramId);
    }

    if !token_account.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    let mint_account =
        Pubkey::create_program_address(&[MINT_SEED, &[global_state.mint_bump()]], program_id)?;

    if token.mint != mint_account {
        return Err(ProgramError::IllegalOwner);
    }

    Ok(())
}
