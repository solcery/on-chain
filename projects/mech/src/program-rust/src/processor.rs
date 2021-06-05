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
use crate::instruction::GrimmzInstruction;
use crate::fight::Fight;
use crate::unit::Unit;

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {


    let instruction = GrimmzInstruction::unpack(instruction_data)?;
    match instruction {
        GrimmzInstruction::CreateCard { data } => {
            msg!("Instruction: CreateCard");
            process_create_card(accounts, data, program_id)
        }
        GrimmzInstruction::Cast { caster_id, target_id } => {
            msg!("Instruction: Cast");
            process_cast(accounts, program_id, caster_id, target_id)
        }
        GrimmzInstruction::CreateFight  => {
            msg!("Instruction: CreateFight");
            process_create_fight(accounts, program_id)
        }
    }
}



pub fn process_create_card(
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: Vec<u8>, // Ignored, all helloworld instructions are hellos
    _program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    let card_account = next_account_info(accounts_iter)?;
    let _mint_account = next_account_info(accounts_iter)?;
    let _payer_account = next_account_info(accounts_iter)?;
    let data = &instruction_data[..]; // Copying instruction_data to mutable slice
    msg!("Slice: {:?}", data);
    let action = Action::try_from_slice(&data).unwrap();
    msg!("Action: {:?}", action);

    action.serialize(&mut &mut card_account.data.borrow_mut()[..])?;
    msg!("{:?}", &mut &mut card_account.data.borrow_mut()[..]);
    msg!("Card account {:?} saved: {:?}", card_account.key, card_account.data);
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
    msg!("fight acc here");
    //let _card_account = next_account_info(accounts_iter)?;
    //msg!("card acc here");
    let card_metadata_account = next_account_info(accounts_iter)?;
    msg!("card_meta acc here");
    msg!("{:?}", &card_metadata_account.data.borrow()[..]);
    let mut action = Action::try_from_slice(&card_metadata_account.data.borrow()[..]).unwrap();
    msg!("Action: {:?}", action);
    msg!("{:?}", &fight_account.data.borrow()[..]);
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
    msg!("{:?}", &mut &mut fight_account.data.borrow_mut()[..]);
    Ok(())
}

