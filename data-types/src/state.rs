use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const CURRENT_GAME_STATE_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub enum Event {
    PlayerUsedObject {
        player_id: u32,
        object_id: u32,
    },
    PlayerUsedObjectOnTarget {
        player_id: u32,
        object_id: u32,
        target_id: u32,
    },
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    log: Vec<Event>,
    state_step: u32,
    game_info: Pubkey,
}

impl State {
    pub fn new(key: Pubkey) -> Self {
        Self {
            log: vec![],
            state_step: 0,
            game_info: key,
        }
    }
}
