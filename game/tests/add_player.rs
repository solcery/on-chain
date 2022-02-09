use solana_program::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    signer::keypair::Keypair,
    system_program,
    transaction::Transaction,
};

use solcery_game::{process_instruction, Instruction as GameInstruction};

#[tokio::test]
async fn add_player() {
    let program_id = Pubkey::new_unique();

    let player_id = Keypair::new();

    let player_info = AccountSharedData::new(1_000, 1024, &player_id.pubkey());
    let (player_info_pda, _) = Pubkey::find_program_address(
        &[b"player", player_id.pubkey().as_ref()],
        &system_program::ID,
    );

    let mut program = ProgramTest::new("solcery_game", program_id, processor!(process_instruction));
    program.add_account(player_info_pda, Account::from(player_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::CreatePlayerAccount,
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new(player_info_pda, false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
