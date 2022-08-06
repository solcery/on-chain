use pretty_assertions::assert_eq;
use solana_program::{
    instruction::Instruction as SolanaInstruction,
    program_pack::Pack,
    pubkey::{Pubkey, PubkeyError},
    system_instruction::create_account,
    system_program::ID as SystemID,
};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    instruction::AccountMeta, program_option::COption, signature::Signer, signer::keypair::Keypair,
    sysvar::rent::ID as RentSysvar, transaction::Transaction,
};
use spl_token::{
    state::{Account as TokenAccount, AccountState, Mint},
    ID as TokenID,
};

use solcery_db_program::{
    entrypoint::process_instruction_bytes,
    instruction::{BootstrapParams, DBInstruction},
    state::{DBGlobalState, GLOBAL_STATE_SEED, MINT_SEED},
};

#[tokio::test]
async fn bootstrap_db() {
    let program_key = Keypair::new();

    let program_id = dbg!(program_key.pubkey());

    let (mint_id, mint_bump) = Pubkey::find_program_address(&[MINT_SEED], &program_id);
    let (global_state_id, state_bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], &program_id);

    let token_key = Keypair::new();
    let token_id = dbg!(token_key.pubkey());

    let program = ProgramTest::new(
        "solcery_db_program",
        program_id,
        processor!(process_instruction_bytes),
    );

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

    let params = BootstrapParams {
        mint_bump,
        state_bump,
        lamports_to_global_state: 5_000_000_000,
        lamports_to_mint: 5_000_000_000,
    };

    let bootstrap_db_program = SolanaInstruction::new_with_borsh(
        program_id,
        &DBInstruction::Bootstrap(params),
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(mint_id, false),
            AccountMeta::new(global_state_id, false),
            AccountMeta::new(token_id, true),
            AccountMeta::new_readonly(SystemID, false),
            AccountMeta::new_readonly(TokenID, false),
            AccountMeta::new(RentSysvar, false),
        ],
    );

    let create_token_account = create_account(
        &admin.pubkey(),
        &token_id,
        5_000_000_000,
        TokenAccount::get_packed_len() as u64,
        &TokenID,
    );

    let mut token_transaction = Transaction::new_with_payer(
        &[create_token_account, bootstrap_db_program],
        Some(&admin.try_pubkey().unwrap()),
    );

    token_transaction.sign(&[&admin, &token_key], recent_blockhash);
    banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap();

    // Retrieving accounts
    let global_state_account = banks_client
        .get_account(global_state_id)
        .await
        .unwrap()
        .unwrap();
    let token_account = banks_client.get_account(token_id).await.unwrap().unwrap();
    let mint_account = banks_client.get_account(mint_id).await.unwrap().unwrap();

    // Deserializing data
    let global_state = DBGlobalState::unpack(&global_state_account.data).unwrap();
    let mint = Mint::unpack(&mint_account.data).unwrap();
    let token = TokenAccount::unpack(&token_account.data).unwrap();

    // Preparing expected data
    let expected_global_state = DBGlobalState::new(state_bump, mint_bump);

    let expected_mint = Mint {
        mint_authority: COption::Some(global_state_id),
        supply: 1,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let expected_token = TokenAccount {
        mint: mint_id,
        owner: admin.pubkey(),
        amount: 1,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    // Assertions
    assert_eq!(global_state, expected_global_state);
    assert_eq!(mint, expected_mint);
    assert_eq!(token, expected_token);
}
