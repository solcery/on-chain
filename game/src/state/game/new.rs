use super::*;
use pretty_assertions::assert_eq;
use solcery_vm::on_chain_types::game_state::GameState;

fn new(
    project_ver: u32,
    correct_project_owner: bool,
    num_players: u32,
    max_items: u32,
    empty_game: bool,
    game_ver: u32,
    empty_game_state: bool,
    game_state_ver: u32,
    expected_result: Result<(), CrateError>,
) {
    let payer_key = Pubkey::new_unique();
    dbg!(&payer_key);
    let player_id = Pubkey::new_unique();
    dbg!(&player_id);
    let program_id = Pubkey::new_unique();
    dbg!(&program_id);
    let game_id = Pubkey::new_unique();
    dbg!(&game_id);
    let game_project_id = Pubkey::new_unique();
    dbg!(&game_project_id);
    let game_state_id = Pubkey::new_unique();
    dbg!(&game_state_id);

    // Game Project preparation
    let mut game_project_balance = 10;
    let mut game_project_data = (
        project_ver,
        Project {
            min_players: 1,
            max_players: 1,
        },
    )
        .try_to_vec()
        .unwrap();

    let game_project_account_info = AccountInfo::new(
        &game_project_id,
        false,
        true,
        &mut game_project_balance,
        &mut game_project_data,
        if correct_project_owner {
            &program_id
        } else {
            &payer_key
        },
        false,
        0,
    );

    // Game preparation
    let mut game_balance = 10;
    let mut game_data = if empty_game {
        vec![0; 10000]
    } else {
        (game_ver, unsafe {
            Game::init(game_project_id, game_state_id, num_players, max_items)
        })
            .try_to_vec()
            .unwrap()
    };

    let game_account_info = AccountInfo::new(
        &game_id,
        false,
        true,
        &mut game_balance,
        &mut game_data,
        &program_id,
        false,
        0,
    );

    // Game State preparation
    let mut game_state_balance = 10;
    let mut game_state_data = if empty_game_state {
        vec![0; 10000]
    } else {
        (game_state_ver, GameState::new()).try_to_vec().unwrap()
    };

    let game_state_account_info = AccountInfo::new(
        &game_state_id,
        false,
        true,
        &mut game_state_balance,
        &mut game_state_data,
        &program_id,
        false,
        0,
    );

    // Game::new() call
    let accounts = vec![
        game_project_account_info,
        game_account_info,
        game_state_account_info,
    ];

    let mut accounts_iter = accounts.iter();

    let result = Game::new(&program_id, &mut accounts_iter, (num_players, max_items));

    if expected_result.is_ok() {
        let expected_game =
            unsafe { Game::init(game_project_id, game_state_id, num_players, max_items) };
        let game_bundle = result.unwrap();
        assert_eq!(game_bundle.data(), &expected_game);
    } else {
        let CrateError = result.unwrap_err();
        assert_eq!(Err(CrateError), expected_result);
    }
}

#[test]
fn correct_input() {
    new(1, true, 1, 0, true, 0, true, 0, Ok(()));
}

#[test]
fn wrong_project_version() {
    new(
        0,
        true,
        1,
        0,
        true,
        0,
        true,
        0,
        Err(CrateError::WrongProjectVersion),
    );
}

#[test]
fn wrong_project_owner() {
    new(
        0,
        false,
        1,
        0,
        true,
        0,
        true,
        0,
        Err(CrateError::WrongAccountOwner),
    );
}

#[test]
fn game_info_already_created() {
    new(
        1,
        true,
        1,
        0,
        false,
        1,
        true,
        0,
        Err(CrateError::AlreadyInUse),
    );
}

#[test]
fn wrong_account_version() {
    new(
        1,
        true,
        1,
        0,
        false,
        2,
        true,
        0,
        Err(CrateError::AlreadyInUse),
    );
}

#[test]
fn already_created_state() {
    new(
        1,
        true,
        1,
        0,
        true,
        0,
        false,
        1,
        Err(CrateError::AlreadyInUse),
    );
}

#[test]
fn wrong_players_number() {
    new(
        1,
        true,
        2,
        0,
        true,
        0,
        true,
        0,
        Err(CrateError::WrongPlayerNumber),
    );
}
