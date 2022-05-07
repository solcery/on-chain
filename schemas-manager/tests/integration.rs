use borsh::BorshDeserialize;
use schemas_manager::processor::{process_instruction_bytes, SchemasManagerInstruction};
use solana_program::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    transaction::Transaction,
};
use solcery_data_types::db::{
    messages::schemas_manager::{AddSchema, GetSchema},
    schema::{AllowedTypes, Schema},
};
use std::str::FromStr;

#[tokio::test]
async fn test_add_schema() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();
    let schema_result = dbg!(Pubkey::new_unique());

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
        tables: vec![AllowedTypes::Int, AllowedTypes::String],
    };

    let mut add_schema_transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &SchemasManagerInstruction::AddSchema {
                message: AddSchema {
                    id: "test_schema_id".to_owned(),
                    schema: new_schema.clone(),
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
            &SchemasManagerInstruction::GetSchema {
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
