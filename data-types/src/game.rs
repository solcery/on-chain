use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Game {
    game_project: Pubkey,
    state_pubkey: Pubkey,
    state_step: u32,
    players: Vec<Player>,
    finished: bool,
    winners: Vec<Pubkey>,
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, BorshSerialize, BorshDeserialize)]
pub struct Player {
    player_id: u32,
    pubkey: Pubkey,
}

#[derive(Eq, PartialEq, Debug, BorshSerialize, BorshDeserialize)]
pub struct GameProject {
    instructions: Pubkey,
    object_types: Pubkey,
    min_players: u32,
    max_players: u32,
    init_game_state: Pubkey,
}
