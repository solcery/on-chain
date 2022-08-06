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

use spl_token::{instruction as token_instruction, state::Mint, ID as TokenID};

mod global_state;

use super::BootstrapParams;
pub use global_state::DBGlobalState;

pub const MINT_SEED: &[u8] = b"DB-program_mint";
pub const GLOBAL_STATE_SEED: &[u8] = b"DB-program_global_state";

pub fn bootstrap<'long: 'short, 'short, AccountIter>(
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
    DBGlobalState::pack(global_state_data, &mut global_state.data.borrow_mut())
}
