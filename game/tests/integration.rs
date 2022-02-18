use borsh::BorshDeserialize;
use pretty_assertions::assert_eq;
use solana_program::{instruction::Instruction as SolanaInstruction, pubkey::Pubkey};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    signature::Signer,
    signer::keypair::Keypair,
    transaction::Transaction,
};

use solcery_data_types::player::{Player, CURRENT_PLAYER_VERSION};
use solcery_game::{process_instruction, Instruction as GameInstruction};

#[tokio::test]
async fn add_player() {
    let program_id = Pubkey::new_unique();

    let player_id = Keypair::new();

    let player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new("solcery_game", program_id, processor!(process_instruction));
    program.add_account(player_info_pda, Account::from(player_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::CreatePlayerAccount,
            vec![
                AccountMeta::new_readonly(dbg!(player_id.pubkey()), true),
                AccountMeta::new(dbg!(player_info_pda), false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);

    banks_client.process_transaction(transaction).await.unwrap();
    let player_info = banks_client
        .get_account(player_info_pda)
        .await
        .unwrap()
        .unwrap();

    let (ver, player): (u32, Player) =
        <(u32, Player)>::deserialize(&mut player_info.data.as_slice()).unwrap();

    let expected_player = unsafe { Player::from_raw_parts(player_id.pubkey(), vec![], None) };

    assert_eq!(ver, CURRENT_PLAYER_VERSION);
    assert_eq!(player, expected_player);
}

#[tokio::test]
#[ignore]
async fn create_game() {
    let program_id = Pubkey::new_unique();

    let player_id = Keypair::new();

    let game_id = Pubkey::new_unique();
    let game_project_id = Pubkey::new_unique();
    let game_state_id = Pubkey::new_unique();

    let player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let game_info = AccountSharedData::new(1_000, 1024, &program_id);
    let game_project = AccountSharedData::new(1_000, 1024, &program_id);
    let game_state = AccountSharedData::new(1_000, 1024, &program_id);

    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new("solcery_game", program_id, processor!(process_instruction));
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));
    program.add_account(game_project_id, Account::from(game_project));
    program.add_account(game_state_id, Account::from(game_state));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[
            SolanaInstruction::new_with_borsh(
                program_id,
                &GameInstruction::CreatePlayerAccount,
                vec![
                    AccountMeta::new_readonly(player_id.pubkey(), true),
                    AccountMeta::new(player_info_pda, false),
                ],
            ),
            SolanaInstruction::new_with_borsh(
                program_id,
                &GameInstruction::CreateGame {
                    num_players: 1,
                    max_items: 0,
                },
                vec![
                    AccountMeta::new_readonly(player_id.pubkey(), true),
                    AccountMeta::new(player_info_pda, false),
                ],
            ),
        ],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
