use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::num::NonZeroU32;

pub const CURRENT_PLAYER_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
//TODO: Add correct Ord implementation
pub struct Player {
    pubkey: Pubkey,
    items: Vec<(u32, Pubkey)>,
    player_id: Option<NonZeroU32>,
}

impl Player {
    #[must_use]
    pub fn from_pubkey(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            items: vec![],
            player_id: None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct PlayerState {
    //TODO: move to SolceryPlayer protocol
    pub pubkey: Pubkey,
    pub game: Option<Pubkey>,
}
