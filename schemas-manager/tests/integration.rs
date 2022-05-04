use borsh::{BorshDeserialize, BorshSerialize};
use schemas_manager::processor::{process_instruction_bytes, SchemasManagerInstruction};
use solana_program::{
    instruction::Instruction as SolanaInstruction, pubkey::Pubkey, system_program,
};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    transaction::Transaction,
};
use solcery_data_types::db::{
    messages::schemas_manager::AddSchema,
    schema::{AllowedTypes, Schema},
};
use spl_token::{
    instruction::{AuthorityType, TokenInstruction},
    state::{Account as TokenAccount, Mint},
    ID as TokenID,
};
use std::str::FromStr;

#[tokio::test]
async fn test_add_schema() {
    let program_id = Pubkey::from_str(&"schemas111111111111111111111111111111111111").unwrap();
    let (allocated_pubkey, _bump_seed) =
        Pubkey::find_program_address(&[b"You pass butter"], &program_id);
    let mut program_test = ProgramTest::new(
        "schemas-manager",
        program_id,
        processor!(process_instruction_bytes),
    );

    let schemas_data = AccountSharedData::new(1_000, 2093, &program_id);

    program_test.add_account(allocated_pubkey, Account::from(schemas_data));

    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;

    let message = AddSchema {
        id: "test_schema_id".to_owned(),
        schema: Schema {
            version: 1u64,
            tables: vec![AllowedTypes::Int, AllowedTypes::String],
        },
    };

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &SchemasManagerInstruction::AddSchema { message },
            vec![
                AccountMeta::new(system_program::id(), false),
                AccountMeta::new(allocated_pubkey, false),
            ],
        )],
        Some(&payer.pubkey()),
    );

    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
