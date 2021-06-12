use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    declare_id,
};
use crate::brick::{
    Action,
    Context,
};
use crate::instruction::SolceryInstruction;
use crate::fight::Fight;
use std::convert::TryInto;
use std::io::Write;


declare_id!("5Ds6QvdZAqwVozdu2i6qzjXm8tmBttV6uHNg4YU8rB1P");

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolceryInstruction::unpack(instruction_data)?;
    match instruction {
        SolceryInstruction::CreateCard { data } => {
            process_create_card(accounts, data, program_id)
        }
        SolceryInstruction::Cast { caster_id, target_id } => {
            process_cast(accounts, program_id, caster_id, target_id)
        }
        SolceryInstruction::CreateFight  => {
            process_create_fight(accounts, program_id)
        }
    }
}


pub fn process_create_card(
    accounts: &[AccountInfo], 
    card_data: Vec<u8>,
    _program_id: &Pubkey, 
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let card_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?; 
    let expected_card_account_pubkey = Pubkey::create_with_seed(
        mint_account.key,
        "SOLCERYCARD",
        &id()
    )?;
    if expected_card_account_pubkey != *card_account.key {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut data = &card_data[..];
    let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap()); // Skipping card visualisation data
    data = &data[client_metadata_size as usize + 4..];
    let _action = Action::try_from_slice(&data[..])?; // 
    let card_account_data = &mut &mut card_account.data.borrow_mut()[..];
    card_account_data.write_all(&card_data[..])?;
    Ok(())
}

pub fn process_cast(
    accounts: &[AccountInfo],
    _program_id: &Pubkey,
    caster_id: u32, // caster unit id (temporary ignored, target is always unit 1)
    target_id: u32, // target unit id (temporary ignored, target is always unit 2)
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?;
    let fight_account = next_account_info(accounts_iter)?;
    let card_metadata_account = next_account_info(accounts_iter)?;
    let client_metadata_size = u32::from_le_bytes(card_metadata_account.data.borrow()[..4].try_into().unwrap());
    let mut action = Action::try_from_slice(&card_metadata_account.data.borrow()[client_metadata_size as usize + 4..]).unwrap();

    let fight = Fight::try_from_slice(&fight_account.data.borrow()[..])?;
    let ctx: &mut Context = &mut Context{ 
         objects: &mut Vec::new(),
    };
    ctx.objects.push(&fight.units[&caster_id]);
    ctx.objects.push(&fight.units[&target_id]);
    action.run(ctx);
    fight.serialize(&mut &mut fight_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_create_fight(
    accounts: &[AccountInfo],
    _program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let fight_account = next_account_info(accounts_iter)?;
    let fight = Fight::new(*payer_account.key); // Pubkey is currently ignored, anybody can cast card in any fight
    fight.serialize(&mut &mut fight_account.data.borrow_mut()[..])?;
    Ok(())
}

