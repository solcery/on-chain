use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

/// Define the type of state stored in accounts
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct GreetingAccount {
    /// number of greetings
    pub number: u32,
}

// Declare and export the program's entrypoint
entrypoint!(process_instruction);

// Program entrypoint's implementation
pub fn process_instruction(
    program_id: &Pubkey, // Public key of the account the program was loaded into
    accounts: &[AccountInfo], // The account to store number in
    instruction_data: &[u8], // Ignored, all helloworld instructions are hellos
) -> ProgramResult {
    msg!("Number memorizer Rust program entrypoint");

    // Iterating accounts is safer then indexing
    let accounts_iter = &mut accounts.iter();

    // Get the account to say hello to
    let account = next_account_info(accounts_iter)?;

    // The account must be owned by the program in order to modify its data
    if account.owner != program_id {
        msg!("Modified account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // Increment and store the number of times the account has been greeted
    let mut greeting_account = GreetingAccount::try_from_slice(&account.data.borrow())?;
    greeting_account.number = unpack_number(instruction_data)?;
    greeting_account.serialize(&mut &mut account.data.borrow_mut()[..])?;

    msg!("Stored number is now {}!", greeting_account.number);

    Ok(())
}

fn unpack_number(input: &[u8]) -> Result<u32, ProgramError> {
    let amount = input
        .get(..4) //get 8 elements from array
        .and_then(|slice| slice.try_into().ok()) // turning it into slice?
        .map(u32::from_le_bytes) // and into u64??
        .ok_or(InvalidInstruction)?;
    Ok(amount)
}
