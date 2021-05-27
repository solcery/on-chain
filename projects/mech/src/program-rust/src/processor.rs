use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    msg,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    
};

use crate::brick;

use crate::fight::{ 
	Fight, 
};

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Greeted account does not have the correct program id");
        return Err(ProgramError::IncorrectProgramId);
    }
    let mut fight = Fight::try_from_slice(&account.data.borrow())?;
    msg!("And now - to fight");
    msg!("fight is{:?}", fight);
    let mut data = &instruction_data[..]; // Copying instruction_data to mutable slice
    let mut action = brick::Action::try_from_slice(&data).unwrap();
    
    fight.serialize(&mut &mut account.data.borrow_mut()[..])?;
    Ok(())
}
