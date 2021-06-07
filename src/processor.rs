use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    
};
use crate::brick::{
    Action,
    Context,
};
use crate::instruction::SolceryInstruction;
use crate::fight::Fight;
use std::convert::TryInto;
use std::io::Write;


entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {


    let instruction = SolceryInstruction::unpack(instruction_data)?;
    match instruction {
        SolceryInstruction::CreateCard { data } => {
            msg!("Instruction: CreateCard");
            process_create_card(accounts, data, program_id)
        }
        SolceryInstruction::Cast { caster_id, target_id } => {
            msg!("Instruction: Cast");
            process_cast(accounts, program_id, caster_id, target_id)
        }
        SolceryInstruction::CreateFight  => {
            msg!("Instruction: CreateFight");
            process_create_fight(accounts, program_id)
        }
    }
}


pub fn process_create_card(
    accounts: &[AccountInfo], // The account to store number in
    card_data: Vec<u8>, // Ignored, all helloworld instructions are hellos
    program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let card_account = next_account_info(accounts_iter)?;
    let _mint_account = next_account_info(accounts_iter)?;
    let _payer_account = next_account_info(accounts_iter)?;
    let mut data = &card_data[..]; // Copying instruction_data to mutable slice

    let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap());
    data = &data[client_metadata_size as usize + 4..];
    let action = Action::try_from_slice(&data[..]).unwrap();
    { 
        let card_account_data = &mut &mut card_account.data.borrow_mut()[..];
        card_account_data.write_all(&card_data[..]);
    }
    Ok(())
}

pub fn process_cast(
    accounts: &[AccountInfo], // The account to store number in
    _program_id: &Pubkey, // Public key of the account the program was loaded into
    caster_id: u8,
    target_id: u8,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let fight_account = next_account_info(accounts_iter)?;
    let card_metadata_account = next_account_info(accounts_iter)?;
    let client_metadata_size = u32::from_le_bytes(card_metadata_account.data.borrow()[..4].try_into().unwrap());
    let mut action = Action::try_from_slice(&card_metadata_account.data.borrow()[client_metadata_size as usize + 4..]).unwrap();
    let mut fight = Fight::try_from_slice(&fight_account.data.borrow()[..])?;
    let ctx: &mut Context = &mut Context{ 
         objects: &mut fight.units,
    };
    action.run(ctx);
    fight.serialize(&mut &mut fight_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_create_fight(
    accounts: &[AccountInfo], // The account to store number in
    _program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let fight_account = next_account_info(accounts_iter)?;
    let fight = Fight::new(*payer_account.key);
    fight.serialize(&mut &mut fight_account.data.borrow_mut()[..])?;
    Ok(())
}

