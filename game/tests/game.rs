use borsh::{BorshDeserialize, BorshSerialize};
use pretty_assertions::assert_eq;
use solana_program::{
    instruction::Instruction as SolanaInstruction, program_pack::Pack, pubkey::Pubkey,
    system_instruction::SystemInstruction, system_program::ID as SystemID,
};
use solana_program_test::{processor, tokio, ProgramTest};
use solana_sdk::{
    account::{Account, AccountSharedData},
    instruction::AccountMeta,
    program_option::COption,
    signature::Signer,
    signer::keypair::Keypair,
    sysvar::rent::ID as RentSysvar,
    transaction::Transaction,
};
use spl_token::{
    instruction::{AuthorityType, TokenInstruction},
    state::{Account as TokenAccount, Mint},
    ID as TokenID,
};
use std::iter;
use std::num::NonZeroU32;

use solcery_game::{
    process_instruction_bytes,
    state::{
        container::Container,
        game::{
            Game, Item, Player as GamePlayer, Project, Status, CURRENT_GAME_PROJECT_VERSION,
            CURRENT_GAME_VERSION,
        },
        player::{Player, CURRENT_PLAYER_VERSION},
        state::{Event, State, CURRENT_GAME_STATE_VERSION},
    },
    Instruction as GameInstruction,
};

#[tokio::test]
async fn create_player() {
    let program_id = Pubkey::new_unique();

    let player_id = Keypair::new();

    let player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
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

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
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
    let expected_state = unsafe { State::init(game_id) };
    let expected_game = unsafe { Game::init(game_project_id, game_state_id, 1, 0) };

    // Assertions
    assert_eq!(game_ver, CURRENT_GAME_VERSION);
    assert_eq!(state_ver, CURRENT_GAME_STATE_VERSION);

    assert_eq!(state, expected_state);
    assert_eq!(game, expected_game);
}

#[tokio::test]
async fn join_game() {
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

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
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

#[tokio::test]
async fn set_status() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_info = AccountSharedData::new(1_000, 1024, &program_id);

    let mut player = Player::from_pubkey(player_id.pubkey());

    unsafe {
        player.set_game(game_id, NonZeroU32::new_unchecked(1));
    }

    let player_data = (CURRENT_PLAYER_VERSION, player);

    let game = unsafe {
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

    let game_data = (CURRENT_GAME_VERSION, game);

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

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::SetGameStatus {
                new_game_status: Status::Started,
            },
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new_readonly(player_info_pda, false),
                AccountMeta::new(game_id, false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Retrieving accounts
    let game_info = banks_client.get_account(game_id).await.unwrap().unwrap();

    // Deserializing data
    let (game_ver, game): (u32, Game) =
        <(u32, Game)>::deserialize(&mut game_info.data.as_slice()).unwrap();

    // Preparing expected data
    let expected_game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Started,
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

    assert_eq!(game, expected_game);
}

#[tokio::test]
async fn leave_game() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_info = AccountSharedData::new(1_000, 1024, &program_id);

    let mut player = Player::from_pubkey(player_id.pubkey());

    unsafe {
        player.set_game(game_id, NonZeroU32::new_unchecked(1));
    }

    let player_data = (CURRENT_PLAYER_VERSION, player);

    let game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Finished { winners: vec![] },
            game_state_id,
            vec![GamePlayer::from_raw_parts(
                NonZeroU32::new_unchecked(1),
                player_id.pubkey(),
                vec![],
            )],
        )
    };

    let game_data = (CURRENT_GAME_VERSION, game);

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

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::LeaveGame,
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
    let (player_ver, player): (u32, Player) =
        <(u32, Player)>::deserialize(&mut player_info.data.as_slice()).unwrap();

    let (game_ver, game): (u32, Game) =
        <(u32, Game)>::deserialize(&mut game_info.data.as_slice()).unwrap();

    // Preparing expected data
    let expected_player = Player::from_pubkey(player_id.pubkey());

    let expected_game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Finished { winners: vec![] },
            game_state_id,
            vec![],
        )
    };

    // Assertions
    assert_eq!(game_ver, CURRENT_GAME_VERSION);
    assert_eq!(player_ver, CURRENT_PLAYER_VERSION);

    assert_eq!(game, expected_game);
    assert_eq!(player, expected_player);
}

#[tokio::test]
async fn add_event() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());
    let container_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut state_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut container_info = AccountSharedData::new(1_000, 1024, &program_id);

    let mut player = Player::from_pubkey(player_id.pubkey());

    unsafe {
        player.set_game(game_id, NonZeroU32::new_unchecked(1));
    }
    let player_data = (CURRENT_PLAYER_VERSION, player);

    let game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Started,
            game_state_id,
            vec![GamePlayer::from_raw_parts(
                NonZeroU32::new_unchecked(1),
                player_id.pubkey(),
                vec![],
            )],
        )
    };
    let game_data = (CURRENT_GAME_VERSION, game);

    let state = unsafe { State::init(game_id) };
    let state_data = (CURRENT_GAME_STATE_VERSION, state);

    let event = Event::PlayerUsedObject {
        player_id: 1,
        object_id: 1,
    };
    let container_data = vec![event];

    // Accounts have to be larger
    let zero_repeater = iter::repeat(0).take(1000);
    let mut player_data = player_data.try_to_vec().unwrap();
    player_data.extend(zero_repeater);

    let zero_repeater = iter::repeat(0).take(1000);
    let mut game_data = game_data.try_to_vec().unwrap();
    game_data.extend(zero_repeater);

    let zero_repeater = iter::repeat(0).take(1000);
    let mut state_data = state_data.try_to_vec().unwrap();
    state_data.extend(zero_repeater);

    let zero_repeater = iter::repeat(0).take(1000);
    let mut container_data = container_data.try_to_vec().unwrap();
    container_data.extend(zero_repeater);

    player_info.set_data(player_data);
    game_info.set_data(game_data);
    state_info.set_data(state_data);
    container_info.set_data(container_data);

    let (player_info_pda, _) =
        Pubkey::find_program_address(&[b"player", player_id.pubkey().as_ref()], &program_id);

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));
    program.add_account(game_state_id, Account::from(state_info));
    program.add_account(container_id, Account::from(container_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::AddEvent {
                state_step: 0,
                event_container: Container::InAccount(container_id),
            },
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new_readonly(player_info_pda, false),
                AccountMeta::new(game_id, false),
                AccountMeta::new(game_state_id, false),
                AccountMeta::new(container_id, false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Retrieving accounts
    let state_info = banks_client
        .get_account(game_state_id)
        .await
        .unwrap()
        .unwrap();

    // Deserializing data
    let (state_ver, state) = <(u32, State)>::deserialize(&mut state_info.data.as_slice()).unwrap();

    // Preparing expected data
    let expected_state = unsafe { State::from_raw_parts(vec![event], 1, game_id) };

    // Assertions
    assert_eq!(state_ver, CURRENT_GAME_STATE_VERSION);

    assert_eq!(state, expected_state);
}

#[tokio::test]
async fn add_item() {
    let player_id = Keypair::new();
    dbg!(player_id.pubkey());

    let mint_id = Keypair::new();
    dbg!(mint_id.pubkey());

    let token_id = Keypair::new();
    dbg!(token_id.pubkey());

    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());

    let mut player_info = AccountSharedData::new(1_000, 1024, &program_id);
    let mut game_info = AccountSharedData::new(1_000, 1024, &program_id);

    let mut player = Player::from_pubkey(player_id.pubkey());

    unsafe {
        player.set_game(game_id, NonZeroU32::new_unchecked(1));
    }
    let player_data = (CURRENT_PLAYER_VERSION, player);

    let game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Initialization {
                remaining_players: 0,
                max_items: 1,
            },
            game_state_id,
            vec![GamePlayer::from_raw_parts(
                NonZeroU32::new_unchecked(1),
                player_id.pubkey(),
                vec![],
            )],
        )
    };
    let game_data = (CURRENT_GAME_VERSION, game);

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

    let mut program = ProgramTest::new(
        "solcery_game",
        program_id,
        processor!(process_instruction_bytes),
    );
    program.add_account(player_info_pda, Account::from(player_info));
    program.add_account(game_id, Account::from(game_info));

    let (mut banks_client, payer, recent_blockhash) = program.start().await;

    // Create and init Mint account
    let init_mint_instruction = TokenInstruction::InitializeMint {
        decimals: 0,
        mint_authority: payer.pubkey(),
        freeze_authority: COption::None,
    };

    let mut mint_transaction = Transaction::new_with_payer(
        &[
            SolanaInstruction::new_with_bincode(
                SystemID,
                &SystemInstruction::CreateAccount {
                    lamports: 5_000_000_000,
                    space: Mint::get_packed_len() as u64,
                    owner: spl_token::ID,
                },
                vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(mint_id.pubkey(), true),
                ],
            ),
            SolanaInstruction::new_with_bytes(
                TokenID,
                &init_mint_instruction.pack(),
                vec![
                    AccountMeta::new(mint_id.pubkey(), false),
                    AccountMeta::new_readonly(RentSysvar, false),
                ],
            ),
        ],
        Some(&payer.try_pubkey().unwrap()),
    );

    mint_transaction.sign(&[&payer, &mint_id], recent_blockhash);
    banks_client
        .process_transaction(mint_transaction)
        .await
        .unwrap();

    // Create, init mint Token account
    let mint_instruction = TokenInstruction::MintTo { amount: 1 };

    // Revoke minting priveleges
    let revoke_instruction = TokenInstruction::SetAuthority {
        authority_type: AuthorityType::MintTokens,
        new_authority: COption::None,
    };
    let mut token_transaction = Transaction::new_with_payer(
        &[
            SolanaInstruction::new_with_bincode(
                SystemID,
                &SystemInstruction::CreateAccount {
                    lamports: 5_000_000_000,
                    space: TokenAccount::get_packed_len() as u64,
                    owner: spl_token::ID,
                },
                vec![
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(token_id.pubkey(), true),
                ],
            ),
            SolanaInstruction::new_with_bytes(
                TokenID,
                &TokenInstruction::InitializeAccount.pack(),
                vec![
                    AccountMeta::new(token_id.pubkey(), false),
                    AccountMeta::new_readonly(mint_id.pubkey(), false),
                    AccountMeta::new_readonly(player_id.pubkey(), false),
                    AccountMeta::new_readonly(RentSysvar, false),
                ],
            ),
            SolanaInstruction::new_with_bytes(
                TokenID,
                &mint_instruction.pack(),
                vec![
                    AccountMeta::new(mint_id.pubkey(), false),
                    AccountMeta::new(token_id.pubkey(), false),
                    AccountMeta::new(payer.pubkey(), true),
                ],
            ),
            SolanaInstruction::new_with_bytes(
                TokenID,
                &revoke_instruction.pack(),
                vec![
                    AccountMeta::new(mint_id.pubkey(), false),
                    AccountMeta::new_readonly(payer.pubkey(), true),
                ],
            ),
        ],
        Some(&payer.try_pubkey().unwrap()),
    );

    token_transaction.sign(&[&payer, &token_id], recent_blockhash);
    banks_client
        .process_transaction(token_transaction)
        .await
        .unwrap();

    let mut transaction = Transaction::new_with_payer(
        &[SolanaInstruction::new_with_borsh(
            program_id,
            &GameInstruction::AddItems { num_items: 1 },
            vec![
                AccountMeta::new_readonly(player_id.pubkey(), true),
                AccountMeta::new(player_info_pda, false),
                AccountMeta::new(game_id, false),
                AccountMeta::new_readonly(token_id.pubkey(), false),
                AccountMeta::new_readonly(mint_id.pubkey(), false),
            ],
        )],
        Some(&payer.try_pubkey().unwrap()),
    );

    transaction.sign(&[&payer, &player_id], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();

    // Retrieving accounts
    let game_info = banks_client.get_account(game_id).await.unwrap().unwrap();

    // Deserializing data
    let (game_ver, game): (u32, Game) =
        <(u32, Game)>::deserialize(&mut game_info.data.as_slice()).unwrap();

    // Preparing expected data
    let expected_game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Initialization {
                remaining_players: 0,
                max_items: 1,
            },
            game_state_id,
            vec![GamePlayer::from_raw_parts(
                NonZeroU32::new_unchecked(1),
                player_id.pubkey(),
                vec![Item::from_raw_parts(
                    NonZeroU32::new(1).unwrap(),
                    token_id.pubkey(),
                )],
            )],
        )
    };

    // Assertions
    assert_eq!(game_ver, CURRENT_GAME_VERSION);

    assert_eq!(game, expected_game);
}
