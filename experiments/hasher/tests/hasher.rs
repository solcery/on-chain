use solana_program::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};

use solana_sdk::{signature::Signer, transaction::Transaction};

use hasher::process_instruction_bytes;

#[tokio::test]
async fn hasher() {
    let program_id = Pubkey::new_unique();
    let program = ProgramTest::new("hasher", program_id, processor!(process_instruction_bytes));

    let (mut banks_client, admin, recent_blockhash) = program.start().await;

    let slice = vec![1; 10];
    let instruction = SolanaInstruction::new_with_bytes(program_id, &slice, vec![]);

    let mut transaction = Transaction::new_with_payer(&[instruction], Some(&admin.pubkey()));

    transaction.sign(&[&admin], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
}
