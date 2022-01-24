use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program_error::ProgramError,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (tag, rest) = instruction_data.split_first().unwrap();
    match tag {
        0 => solcery_template::process_instruction(accounts, rest)?,
        1 => solcery_object::process_instruction(accounts, rest)?,
        2 => solcery_storage::process_instruction(accounts, rest)?,
        3 => solcery_crud::process_instruction(accounts, rest)?,
        4 => solcery_project::process_instruction(accounts, rest)?,
        _ => return Err(ProgramError::InvalidAccountData),
    }
    Ok(())
}
