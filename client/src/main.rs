use borsh::ser::BorshSerialize;
use solana_client::rpc_client::RpcClient;
use solana_program::{
    instruction::Instruction as SolanaInstruction, instruction::InstructionError,
    program_pack::Pack, pubkey::Pubkey, system_instruction::create_account,
    system_program::ID as SystemID,
};
use solana_sdk::{
    account::{Account, AccountSharedData},
    account_info::AccountInfo,
    instruction::AccountMeta,
    program_option::COption,
    signature::Signer,
    signer::keypair::Keypair,
    sysvar::rent::ID as RentSysvar,
    transaction::{Transaction, TransactionError},
};
use spl_token::{
    state::{Account as TokenAccount, AccountState, Mint},
    ID as TokenID,
};
use std::cell::RefCell;
use std::rc::Rc;

use solcery_db::DB;
use solcery_db_program::{
    instruction::*,
    state::{DBGlobalState, GLOBAL_STATE_SEED, MINT_SEED},
};

const AMOUNT: u64 = 1_000_000_000;
fn main() {
    let admin = Keypair::new();
    let token_key = Keypair::new();

    let token_id = token_key.pubkey();
    let program_id = Pubkey::try_from("6HT39VNwNJFuKPFkHchwiRnxDx157ppwMJ618jzRLNb1").unwrap();
    println!("Hello, world!{}", program_id);

    let new_token_key = Keypair::new();
    let new_token_id = new_token_key.pubkey();
    println!("New token: {}", new_token_key.to_base58_string());

    let (mint_id, mint_bump) = Pubkey::find_program_address(&[MINT_SEED], &program_id);
    let (global_state_id, state_bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], &program_id);

    let mint_new_token = SolanaInstruction::new_with_borsh(
        program_id,
        &DBInstruction::MintNewAccessToken,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(mint_id, false),
            AccountMeta::new(global_state_id, false),
            AccountMeta::new(token_id, true),
            AccountMeta::new(new_token_id, true),
            AccountMeta::new_readonly(TokenID, false),
            AccountMeta::new(RentSysvar, false),
        ],
    );

    let create_token_account = create_account(
        &admin.pubkey(),
        &new_token_id,
        AMOUNT,
        TokenAccount::get_packed_len() as u64,
        &TokenID,
    );

    let mut token_transaction = Transaction::new_with_payer(
        &[create_token_account, mint_new_token],
        Some(&admin.try_pubkey().unwrap()),
    );

    let url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new(url);
    let recent_blockhash = client.get_latest_blockhash().unwrap();

    token_transaction.sign(&[&admin, &token_key, &new_token_key], recent_blockhash);

    let signature = client.send_transaction(&token_transaction).unwrap();

    dbg!(signature);
}
