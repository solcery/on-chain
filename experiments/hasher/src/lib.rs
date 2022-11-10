use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, hash::Hasher,
    log::sol_log_compute_units, pubkey::Pubkey,
};

entrypoint!(process_instruction_bytes);
pub fn process_instruction_bytes(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    println!("1");
    sol_log_compute_units();
    let mut hasher = Hasher::default();
    hasher.hash(instruction_data);
    sol_log_compute_units();

    let _hash = hasher.result();
    Ok(())
}
