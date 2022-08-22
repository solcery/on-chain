use db_manager::processor::{process_instruction_bytes, DataBaseInstruction};
use solana_program::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    transaction::Transaction,
};
use solcery_data_types::db::{
    messages::schemas_manager::AddSchema,
    schema::{AllowedTypes, KeyType, Schema},
};
use std::str::FromStr;

#[tokio::test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: TransactionError(InstructionError(0, Custom(3)))"
)]
async fn test_add_schema_incorrect_primary_key_count_more_than_one() {
    let program_id = Pubkey::from_str("schemas111111111111111111111111111111111111").unwrap();

    let (app_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut schemas_manager_app = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_holder_data = AccountSharedData::new(1_000, 2093, &program_id);
    schemas_manager_app.add_account(app_pubkey, Account::from(schemas_holder_data));

    let (mut banks_client, payer, recent_blockhash) = schemas_manager_app.start().await;

    // Add

    let incorrect_schema = Schema {
        version: 1u64,
        tables: vec![
            AllowedTypes::Int(KeyType::Primary),
            AllowedTypes::String(KeyType::Primary),
        ],
    };

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: incorrect_schema.clone(),
                    need_init: true,
                },
            },
            vec![AccountMeta::new(app_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );

    add_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(add_schema_transaction)
        .await
        .unwrap();
}

#[tokio::test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: TransactionError(InstructionError(0, Custom(3)))"
)]
async fn test_add_schema_incorrect_primary_key_count_less_than_one() {
    let program_id = Pubkey::from_str("schemas111111111111111111111111111111111111").unwrap();

    let (app_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut schemas_manager_app = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_holder_data = AccountSharedData::new(1_000, 2093, &program_id);
    schemas_manager_app.add_account(app_pubkey, Account::from(schemas_holder_data));

    let (mut banks_client, payer, recent_blockhash) = schemas_manager_app.start().await;

    // Add

    let incorrect_schema = Schema {
        version: 1u64,
        tables: vec![
            AllowedTypes::Int(KeyType::NotKey),
            AllowedTypes::String(KeyType::NotKey),
        ],
    };

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: incorrect_schema.clone(),
                    need_init: true,
                },
            },
            vec![AccountMeta::new(app_pubkey, false)],
        )],
        Some(&payer.pubkey()),
    );

    add_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(add_schema_transaction)
        .await
        .unwrap();
}
