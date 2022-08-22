use pretty_assertions::assert_eq;
use solana_program::{
    instruction::Instruction as SolanaInstruction, instruction::InstructionError,
    program_pack::Pack, pubkey::Pubkey, system_instruction::create_account,
    system_program::ID as SystemID,
};
use solana_program_test::{processor, tokio, ProgramTest};
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

use account_fs::FS;
use solcery_db::DB;
use solcery_db_program::{
    entrypoint::process_instruction_bytes,
    instruction::*,
    state::{DBGlobalState, GLOBAL_STATE_SEED, MINT_SEED},
};

const AMOUNT: u64 = 5_000_000_000;
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
        lamports_to_global_state: AMOUNT,
        lamports_to_mint: AMOUNT,
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
        AMOUNT,
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

#[tokio::test]
async fn try_rebootstrap() {
    let ProgramEnvironment {
        global_state: global_state_id,
        mint: mint_id,
        program: program_key,
        test: program,
        token: token_key,
        mint_bump,
        state_bump,
        ..
    } = prepare_environment();

    let program_id = program_key.pubkey();

    let new_token_key = Keypair::new();
    let new_token_id = new_token_key.pubkey();

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

    let params = BootstrapParams {
        mint_bump,
        state_bump,
        lamports_to_global_state: AMOUNT,
        lamports_to_mint: AMOUNT,
    };

    let bootstrap_db_program = SolanaInstruction::new_with_borsh(
        program_id,
        &DBInstruction::Bootstrap(params),
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(mint_id, false),
            AccountMeta::new(global_state_id, false),
            AccountMeta::new(new_token_id, true),
            AccountMeta::new_readonly(SystemID, false),
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
        &[create_token_account, bootstrap_db_program],
        Some(&admin.try_pubkey().unwrap()),
    );

    token_transaction.sign(&[&admin, &new_token_key], recent_blockhash);

    let result = banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap_err();

    assert_eq!(
        result.unwrap(),
        TransactionError::InstructionError(1, InstructionError::Custom(0))
    );
}

#[tokio::test]
async fn mint_new_access_token() {
    let ProgramEnvironment {
        global_state: global_state_id,
        mint: mint_id,
        program: program_key,
        test: program,
        token: token_key,
        ..
    } = prepare_environment();

    let program_id = program_key.pubkey();
    let token_id = token_key.pubkey();

    let new_token_key = Keypair::new();
    let new_token_id = new_token_key.pubkey();

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

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

    token_transaction.sign(&[&admin, &token_key, &new_token_key], recent_blockhash);
    banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap();

    // Retrieving accounts
    let token_account = banks_client
        .get_account(new_token_id)
        .await
        .unwrap()
        .unwrap();
    let mint_account = banks_client.get_account(mint_id).await.unwrap().unwrap();

    // Deserializing data
    let mint = Mint::unpack(&mint_account.data).unwrap();
    let token = TokenAccount::unpack(&token_account.data).unwrap();

    // Preparing expected data
    let expected_mint = Mint {
        mint_authority: COption::Some(global_state_id),
        supply: 2,
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
    assert_eq!(mint, expected_mint);
    assert_eq!(token, expected_token);
}

#[tokio::test]
async fn try_mint_new_unsigned_token() {
    let ProgramEnvironment {
        global_state: global_state_id,
        mint: mint_id,
        program: program_key,
        test: program,
        token: token_key,
        ..
    } = prepare_environment();

    let program_id = program_key.pubkey();
    let token_id = token_key.pubkey();

    let new_token_key = Keypair::new();
    let new_token_id = new_token_key.pubkey();

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

    let mint_new_token = SolanaInstruction::new_with_borsh(
        program_id,
        &DBInstruction::MintNewAccessToken,
        vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(mint_id, false),
            AccountMeta::new(global_state_id, false),
            AccountMeta::new(token_id, false),
            AccountMeta::new(new_token_id, true),
            AccountMeta::new_readonly(TokenID, false),
            AccountMeta::new(RentSysvar, false),
        ],
    );

    let create_token_account = create_account(
        &admin.pubkey(),
        &new_token_id,
        5_000_000_000,
        TokenAccount::get_packed_len() as u64,
        &TokenID,
    );

    let mut token_transaction = Transaction::new_with_payer(
        &[create_token_account, mint_new_token],
        Some(&admin.try_pubkey().unwrap()),
    );

    token_transaction.sign(&[&admin, &new_token_key], recent_blockhash);

    let result = banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap_err();

    assert_eq!(
        result.unwrap(),
        TransactionError::InstructionError(1, InstructionError::MissingRequiredSignature)
    );
}

#[tokio::test]
async fn create_db() {
    let ProgramEnvironment {
        global_state: global_state_id,
        mint: mint_id,
        program: program_key,
        test: program,
        token: token_key,
        ..
    } = prepare_environment();

    let program_id = program_key.pubkey();
    let token_id = token_key.pubkey();

    let fs_account_key = Keypair::new();

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

    let create_fs_account = create_account(
        &admin.pubkey(),
        &fs_account_key.pubkey(),
        AMOUNT * 2,
        1_000_000,
        &program_id,
    );

    let db_instruction = DBInstruction::CreateDB(CreateDBParams {
        primary_key_type: DataType::Int,
        columns: vec![],
        table_name: String::from("Test DB"),
        max_columns: 10,
        max_rows: 10,
        is_initialized: false,
    });

    let instruction = SolanaInstruction::new_with_borsh(
        program_id,
        &db_instruction,
        vec![
            AccountMeta::new_readonly(global_state_id, false),
            AccountMeta::new_readonly(token_id, true),
            AccountMeta::new(fs_account_key.pubkey(), false),
        ],
    );

    let mut token_transaction =
        Transaction::new_with_payer(&[create_fs_account, instruction], Some(&admin.pubkey()));

    token_transaction.sign(&[&admin, &token_key, &fs_account_key], recent_blockhash);

    banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap();

    // Retrieving accounts
    let fs_account = banks_client
        .get_account(fs_account_key.pubkey())
        .await
        .unwrap()
        .unwrap();

    let mut account_internals = (fs_account_key.pubkey(), fs_account);
    let fs_account = vec![AccountInfo::from(&mut account_internals)];

    // Deserializing data
    let fs = Rc::new(RefCell::new(
        FS::from_account_iter(&program_id, &mut fs_account.iter()).unwrap(),
    ));

    DB::from_segment(
        fs,
        SegmentId {
            id: 0,
            pubkey: fs_account_key.pubkey(),
        },
    )
    .unwrap();
}

struct ProgramEnvironment {
    global_state: Pubkey,
    mint: Pubkey,
    program: Keypair,
    test: ProgramTest,
    token: Keypair,
    user: Keypair,
    mint_bump: u8,
    state_bump: u8,
}

fn prepare_environment() -> ProgramEnvironment {
    let program_key = Keypair::new();
    let user = Keypair::new();

    let program_id = program_key.pubkey();

    let (mint_id, mint_bump) = Pubkey::find_program_address(&[MINT_SEED], &program_id);
    let (global_state_id, state_bump) =
        Pubkey::find_program_address(&[GLOBAL_STATE_SEED], &program_id);

    let token_key = Keypair::new();
    let token_id = token_key.pubkey();

    let global_state_data = DBGlobalState::new(state_bump, mint_bump);

    let mut global_state =
        AccountSharedData::new(AMOUNT, DBGlobalState::get_packed_len(), &program_id);

    let mut data = vec![0; DBGlobalState::get_packed_len()];

    DBGlobalState::pack(global_state_data, &mut data).unwrap();

    global_state.set_data(data);

    let mint = Mint {
        mint_authority: COption::Some(global_state_id),
        supply: 1,
        decimals: 0,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut data = vec![0; Mint::get_packed_len()];
    Mint::pack(mint, &mut data).unwrap();

    let mut mint = AccountSharedData::new(AMOUNT, Mint::get_packed_len(), &TokenID);

    mint.set_data(data);

    let token = TokenAccount {
        mint: mint_id,
        owner: user.pubkey(),
        amount: 1,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut data = vec![0; TokenAccount::get_packed_len()];
    TokenAccount::pack(token, &mut data).unwrap();

    let mut token = AccountSharedData::new(AMOUNT, TokenAccount::get_packed_len(), &TokenID);

    token.set_data(data);

    let mut program = ProgramTest::new(
        "solcery_db_program",
        program_id,
        processor!(process_instruction_bytes),
    );

    program.add_account(mint_id, Account::from(mint));
    program.add_account(token_id, Account::from(token));
    program.add_account(global_state_id, Account::from(global_state));

    ProgramEnvironment {
        global_state: global_state_id,
        mint: mint_id,
        program: program_key,
        test: program,
        token: token_key,
        user,
        mint_bump,
        state_bump,
    }
}
