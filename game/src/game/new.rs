use super::*;
use pretty_assertions::assert_eq;

fn new(num_players: u32, max_items: u32, expected_result: Result<(), Error>) {
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
        CURRENT_GAME_PROJECT_VERSION,
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
        &program_id,
        false,
        0,
    );

    // Game preparation
    let mut game_balance = 10;
    let mut game_data = vec![0; 10000];

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
    let mut game_state_data = vec![0; 10000];

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
        let error = result.unwrap_err();
        assert_eq!(Err(error), expected_result);
    }
}

#[test]
fn correct_input() {
    new(1, 0, Ok(()));
}
