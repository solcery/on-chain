use borsh::{BorshDeserialize, BorshSerialize};
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
use std::iter;
use std::num::NonZeroU32;

use solcery_data_types::{
    game::{
        Game, Player as GamePlayer, Project, Status, CURRENT_GAME_PROJECT_VERSION,
        CURRENT_GAME_VERSION,
    },
    player::{Player, CURRENT_PLAYER_VERSION},
    state::{State, CURRENT_GAME_STATE_VERSION},
};
use solcery_game::{process_instruction, Instruction as GameInstruction};

#[tokio::test]
async fn create_player() {
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

    let expected_player = Player::from_pubkey(player_id.pubkey());

    assert_eq!(ver, CURRENT_PLAYER_VERSION);
    assert_eq!(player, expected_player);
}

#[tokio::test]
async fn create_game() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let game_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_project = AccountSharedData::new(1_000, 1024, &program_id);
    let game_state = AccountSharedData::new(1_000, 1024, &program_id);

    let game_project_data = (
        CURRENT_GAME_PROJECT_VERSION,
        Project {
            min_players: 1,
            max_players: 1,
        },
    );

    let player_data = (
        CURRENT_PLAYER_VERSION,
        Player::from_pubkey(player_id.pubkey()),
    );

    game_project.set_data(game_project_data.try_to_vec().unwrap());
    player_info.set_data(player_data.try_to_vec().unwrap());

    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new("solcery_game", program_id, processor!(process_instruction));
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));
    program.add_account(game_project_id, Account::from(game_project));
    program.add_account(game_state_id, Account::from(game_state));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::CreateGame {
                num_players: 1,
                max_items: 0,
            },
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new_readonly(player_info_pda, false),
                AccountMeta::new_readonly(game_project_id, false),
                AccountMeta::new(game_id, false),
                AccountMeta::new(game_state_id, false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Retrieving accounts
    let game_info = banks_client.get_account(game_id).await.unwrap().unwrap();

    let state_info = banks_client
        .get_account(game_state_id)
        .await
        .unwrap()
        .unwrap();

    // Deserializing data
    let (game_ver, game): (u32, Game) =
        <(u32, Game)>::deserialize(&mut game_info.data.as_slice()).unwrap();

    let (state_ver, state) = <(u32, State)>::deserialize(&mut state_info.data.as_slice()).unwrap();

    // Preparing expected data
    let expected_state = State::init(game_id);
    let expected_game = unsafe { Game::init(game_project_id, game_state_id, 1, 0) };

    // Assertions
    assert_eq!(game_ver, CURRENT_GAME_VERSION);
    assert_eq!(state_ver, CURRENT_GAME_STATE_VERSION);

    assert_eq!(state, expected_state);
    assert_eq!(game, expected_game);
}

#[tokio::test]
async fn add_player() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_info = AccountSharedData::new(1_000, 1024, &program_id);

    let player_data = (
        CURRENT_PLAYER_VERSION,
        Player::from_pubkey(player_id.pubkey()),
    );

    let game_data = (CURRENT_GAME_VERSION, unsafe {
        Game::init(game_project_id, game_state_id, 1, 0)
    });

    // Accounts have to be larger
    let zero_repeater = iter::repeat(0).take(1000);
    let mut player_data = player_data.try_to_vec().unwrap();
    player_data.extend(zero_repeater);

    let zero_repeater = iter::repeat(0).take(1000);
    let mut game_data = game_data.try_to_vec().unwrap();
    game_data.extend(zero_repeater);

    player_info.set_data(player_data);
    game_info.set_data(game_data);

    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new("solcery_game", program_id, processor!(process_instruction));
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::JoinGame,
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new(player_info_pda, false),
                AccountMeta::new(game_id, false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Retrieving accounts
    let player_info = banks_client
        .get_account(player_info_pda)
        .await
        .unwrap()
        .unwrap();

    let game_info = banks_client.get_account(game_id).await.unwrap().unwrap();

    // Deserializing data
    let (game_ver, game): (u32, Game) =
        <(u32, Game)>::deserialize(&mut game_info.data.as_slice()).unwrap();

    let (player_ver, player): (u32, Player) =
        <(u32, Player)>::deserialize(&mut player_info.data.as_slice()).unwrap();

    // Preparing expected data
    let mut expected_player = Player::from_pubkey(player_id.pubkey());

    unsafe {
        expected_player.set_game(game_id, NonZeroU32::new_unchecked(1));
    }

    let expected_game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Initialization {
                remaining_players: 0,
                max_items: 0,
            },
            game_state_id,
            vec![GamePlayer::from_raw_parts(
                NonZeroU32::new_unchecked(1),
                player_id.pubkey(),
                vec![],
            )],
        )
    };

    // Assertions
    assert_eq!(game_ver, CURRENT_GAME_VERSION);
    assert_eq!(player_ver, CURRENT_PLAYER_VERSION);

    assert_eq!(game, expected_game);
    assert_eq!(player, expected_player);
}
