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
};
use crate::instruction::GrimmzInstruction;

entrypoint!(process_instruction);
pub fn process_instruction(


    program_id: &Pubkey, // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {


    let instruction = GrimmzInstruction::unpack(instruction_data)?;
    match instruction {
        GrimmzInstruction::CreateCard { data } => {
            msg!("Instruction: CreateCard");
            process_create_card(accounts, data, program_id)
        }
        GrimmzInstruction::Execute => {
            msg!("Instruction: Execute");
            process_execute(accounts, program_id)
        }
    }
}



pub fn process_create_card(
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: Vec<u8>, // Ignored, all helloworld instructions are hellos
    _program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {

    let accounts_iter = &mut accounts.iter();
    msg!("Process instruction");
    
    // Get the account to say hello to
    let card_account = next_account_info(accounts_iter)?;
    let _mint_account = next_account_info(accounts_iter)?;
    let _payer_account = next_account_info(accounts_iter)?;
    let data = &instruction_data[..]; // Copying instruction_data to mutable slice
    let action = Action::try_from_slice(&data).unwrap();
    msg!("Action: {:?}", action);

    action.serialize(&mut &mut card_account.data.borrow_mut()[..])?;

    msg!("Card account {:?} saved: {:?}", card_account.key, card_account.data);
    Ok(())
}

pub fn process_execute(
    accounts: &[AccountInfo], // The account to store number in
    _program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    msg!("Process instruction");
    
    // Get the account to say hello to
    let _mint_account = next_account_info(accounts_iter)?;
    let _card_metadata_account = next_account_info(accounts_iter)?;

    // let expected_pubkey = Pubkey::create_with_seed(
    //     mint_account,
    //     "CREATE CARD",
    //     program_id,
    // );

    // if card_metadata_account.key != expected_pubkey {
    //      return Err(ProgramError);
    // }

    //let mut action = brick::Action::try_from_slice(&mut mint_account.data.borrow()[..])?;
    msg!("Action deserialized: {:?}");
    Ok(())
}

