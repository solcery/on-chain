use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, pubkey::Pubkey,
};

use super::processor;

entrypoint!(process_instruction_bytes);
pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    processor::process_instruction_bytes(program_id, accounts, instruction_data)
}
