use borsh::BorshDeserialize;
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
    messages::schemas_manager::{AddSchema, GetSchema, RemoveSchema, UpdateSchema},
    schema::{AllowedTypes, KeyType, Schema, Tables},
};
use std::str::FromStr;

#[tokio::test]
async fn test_add_schema() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();
    let schema_result = Pubkey::new_unique();

    let (app_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut schemas_manager_app = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_holder_data = AccountSharedData::new(1_000, 2093, &program_id);
    schemas_manager_app.add_account(app_pubkey, Account::from(schemas_holder_data));

    let schema_result_data = AccountSharedData::new(1_000, 16, &program_id);
    schemas_manager_app.add_account(schema_result, Account::from(schema_result_data));

    let (mut banks_client, payer, recent_blockhash) = schemas_manager_app.start().await;

    // Add

    let new_schema = Schema {
        version: 1u64,
        tables: vec![
            AllowedTypes::Int(KeyType::Primary),
            AllowedTypes::String(KeyType::NotKey),
        ],
    };

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: new_schema.clone(),
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

    // Get

    let mut get_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::GetSchema {
                message: GetSchema {
                    id: "test_schema_id".to_owned(),
                },
            },
            vec![
                AccountMeta::new(app_pubkey, false),
                AccountMeta::new(schema_result, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    get_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(get_schema_transaction)
        .await
        .unwrap();

    // Check

    let schema_info = banks_client
        .get_account(schema_result)
        .await
        .unwrap()
        .unwrap();
    let schema: Schema = <Schema>::deserialize(&mut schema_info.data.as_slice()).unwrap();

    assert_eq!(new_schema, schema);
}

#[tokio::test]
#[should_panic(
    expected = "called `Result::unwrap()` on an `Err` value: TransactionError(InstructionError(0, Custom(3)))"
)]
async fn test_add_schema_incorrect_primary_key_count_more_than_one() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();

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
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();

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

#[tokio::test]
async fn test_remove_schema() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();
    let schema_result = Pubkey::new_unique();

    let (app_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut schemas_manager_app = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_holder_data = AccountSharedData::new(1_000, 2093, &program_id);
    schemas_manager_app.add_account(app_pubkey, Account::from(schemas_holder_data));

    let schema_result_data = AccountSharedData::new(1_000, 14, &program_id);
    schemas_manager_app.add_account(schema_result, Account::from(schema_result_data));

    let (mut banks_client, payer, recent_blockhash) = schemas_manager_app.start().await;

    // Add

    let new_schema = Schema {
        version: 1u64,
        tables: vec![
            AllowedTypes::Int(KeyType::Primary),
            AllowedTypes::String(KeyType::NotKey),
        ],
    };

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: new_schema.clone(),
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

    // Remove

    let mut remove_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::RemoveSchema {
                message: RemoveSchema {
                    id: "test_schema_id".to_owned(),
                },
            },
            vec![
                AccountMeta::new(app_pubkey, false),
                AccountMeta::new(schema_result, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    remove_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(remove_schema_transaction)
        .await
        .unwrap();

    // Check

    let mut get_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::GetSchema {
                message: GetSchema {
                    id: "test_schema_id".to_owned(),
                },
            },
            vec![
                AccountMeta::new(app_pubkey, false),
                AccountMeta::new(schema_result, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    get_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(get_schema_transaction)
        .await
        .unwrap();

    let schema_info = banks_client
        .get_account(schema_result)
        .await
        .unwrap()
        .unwrap();
    let schema: Schema = <Schema>::deserialize(&mut schema_info.data.as_slice()).unwrap();

    assert_ne!(new_schema, schema);
}

#[tokio::test]
async fn test_update_schema() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();
    let schema_result = Pubkey::new_unique();

    let (app_pubkey, _bump_seed) = Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut schemas_manager_app = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_holder_data = AccountSharedData::new(1_000, 2093, &program_id);
    schemas_manager_app.add_account(app_pubkey, Account::from(schemas_holder_data));

    let schema_result_data = AccountSharedData::new(1_000, 20, &program_id);
    schemas_manager_app.add_account(schema_result, Account::from(schema_result_data));

    let (mut banks_client, payer, recent_blockhash) = schemas_manager_app.start().await;

    // Add

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: Schema {
                        version: 1u64,
                        tables: vec![
                            AllowedTypes::Int(KeyType::Primary),
                            AllowedTypes::String(KeyType::NotKey),
                        ],
                    },
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

    // Update

    let new_tables: Tables = vec![
        AllowedTypes::Int(KeyType::Primary),
        AllowedTypes::Int(KeyType::NotKey),
        AllowedTypes::String(KeyType::NotKey),
        AllowedTypes::String(KeyType::NotKey),
    ];

    let mut update_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::UpdateSchema {
                message: UpdateSchema {
                    id: "test_schema_id".to_owned(),
                    tables: new_tables.clone(),
                },
            },
            vec![
                AccountMeta::new(app_pubkey, false),
                AccountMeta::new(schema_result, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    update_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(update_schema_transaction)
        .await
        .unwrap();

    // Check

    let mut get_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &DataBaseInstruction::GetSchema {
                message: GetSchema {
                    id: "test_schema_id".to_owned(),
                },
            },
            vec![
                AccountMeta::new(app_pubkey, false),
                AccountMeta::new(schema_result, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    get_schema_transaction.sign(&[&payer], recent_blockhash);
    banks_client
        .process_transaction(get_schema_transaction)
        .await
        .unwrap();

    let schema_info = banks_client
        .get_account(schema_result)
        .await
        .unwrap()
        .unwrap();
    let schema: Schema = <Schema>::deserialize(&mut schema_info.data.as_slice()).unwrap();

    assert_eq!(schema.tables, new_tables);
    assert_eq!(schema.version, 2u64);
}
