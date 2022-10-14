use super::*;
use crate::state::{game::Player as GamePlayer, player::Player};
use pretty_assertions::assert_eq;
use solana_sdk::program_option::COption;
use spl_token::state::{Account as TokenAccount, AccountState, Mint};
use std::num::NonZeroU32;

fn add_item(
    correct_mint: bool,
    correct_owner: bool,
    authority_present: bool,
    supply: u64,
    decimals: u8,
    in_game: bool,
    expected_result: Result<(), Error>,
) {
    let payer_key = dbg!(Pubkey::new_unique());
    let player_id = dbg!(Pubkey::new_unique());
    let program_id = dbg!(Pubkey::new_unique());
    let game_id = dbg!(Pubkey::new_unique());
    let game_project_id = dbg!(Pubkey::new_unique());
    let game_state_id = dbg!(Pubkey::new_unique());
    let mint_id = dbg!(Pubkey::new_unique());
    let token_id = dbg!(Pubkey::new_unique());

    // Token account preparation
    let token = TokenAccount {
        mint: if correct_mint { mint_id } else { payer_key },
        owner: if correct_owner { player_id } else { payer_key },
        amount: 1,
        delegate: COption::None,
        state: AccountState::Initialized,
        is_native: COption::None,
        delegated_amount: 0,
        close_authority: COption::None,
    };

    let mut token_bytes = vec![0; TokenAccount::get_packed_len()];
    TokenAccount::pack(token, &mut token_bytes).unwrap();

    let mut token_balance = 10;
    let token_account_info = AccountInfo::new(
        &token_id,
        false,
        true,
        &mut token_balance,
        &mut token_bytes,
        &spl_token::ID,
        false,
        0,
    );

    // Mint account preparation
    let mint = Mint {
        mint_authority: if authority_present {
            COption::Some(game_project_id)
        } else {
            COption::None
        },
        supply,
        decimals,
        is_initialized: true,
        freeze_authority: COption::None,
    };

    let mut mint_bytes = vec![0; Mint::get_packed_len()];
    Mint::pack(mint, &mut mint_bytes).unwrap();

    let mut mint_balance = 10;
    let mint_account_info = AccountInfo::new(
        &mint_id,
        false,
        true,
        &mut mint_balance,
        &mut mint_bytes,
        &spl_token::ID,
        false,
        0,
    );

    // Player preparation
    let (pda, _) = Pubkey::find_program_address(&[b"player", payer_key.as_ref()], &program_id);

    let mut player = Player::from_pubkey(player_id);

    if in_game {
        unsafe {
            player.set_game(game_id, NonZeroU32::new_unchecked(1));
        }
    }

    // Dummy values, won't be used.
    let mut player_balance = 10;
    let mut player_data = vec![0; 10];

    let player_account_info = AccountInfo::new(
        &pda,
        false,
        true,
        &mut player_balance,
        &mut player_data,
        &program_id,
        false,
        0,
    );

    let player = unsafe { Bundled::<Player>::new(player, &player_account_info) };

    // Game preparation
    let players = if in_game {
        vec![unsafe { GamePlayer::from_raw_parts(NonZeroU32::new_unchecked(1), player_id, vec![]) }]
    } else {
        vec![]
    };

    let game = unsafe {
        Game::from_raw_parts(
            game_project_id,
            Status::Initialization {
                remaining_players: 0,
                max_items: 1,
            },
            game_state_id,
            players,
        )
    };

    // Dummy values, won't be used.
    let mut game_balance = 10;
    let mut game_data = vec![0; 10];

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

    let mut game = unsafe { Bundled::<Game>::new(game, &game_account_info) };

    let result = game.add_items(&player, vec![(&token_account_info, &mint_account_info)]);

    assert_eq!(result, expected_result);
}

#[test]
fn correct_token() {
    add_item(true, true, false, 1, 0, true, Ok(()));
}

#[test]
fn not_in_game() {
    add_item(true, true, false, 1, 0, false, Err(Error::NotInGame));
}

#[test]
fn wrong_account_mint() {
    add_item(false, true, false, 1, 0, true, Err(Error::WrongAccountMint));
}

#[test]
fn not_owned_nft() {
    add_item(true, false, false, 1, 0, true, Err(Error::NotOwnedNFT));
}

#[test]
fn authority_present() {
    add_item(true, true, true, 1, 0, true, Err(Error::NotAnNFT));
}

#[test]
fn oversupplied() {
    add_item(true, true, false, 2, 0, true, Err(Error::NotAnNFT));
}

#[test]
fn decimals() {
    add_item(true, true, false, 1, 2, true, Err(Error::NotAnNFT));
}
