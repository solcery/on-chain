use borsh::{BorshDeserialize, BorshSerialize};

use solana_program::pubkey::Pubkey;
use std::num::NonZeroU32;

pub const CURRENT_PLAYER_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
//TODO: Add correct Ord implementation
pub struct Player {
    pubkey: Pubkey,
    items: Vec<(u32, Pubkey)>,
    game: Option<GameInfo>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct GameInfo {
    player_id: NonZeroU32,
    game_key: Pubkey,
}

impl Player {
    #[must_use]
    pub fn from_pubkey(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            items: vec![],
            game: None,
        }
    }

    #[must_use]
    pub unsafe fn from_raw_parts(
        pubkey: Pubkey,
        items: Vec<(u32, Pubkey)>,
        game: Option<GameInfo>,
    ) -> Self {
        Self {
            pubkey,
            items,
            game,
        }
    }

    #[must_use]
    pub fn key(&self) -> Pubkey {
        self.pubkey
    }

    #[must_use]
    pub fn in_game(&self) -> bool {
        self.game.is_some()
    }

    #[must_use]
    pub fn game_key(&self) -> Option<Pubkey> {
        self.game.as_ref().map(|game| game.game_key)
    }

    pub unsafe fn set_game(&mut self, game_key: Pubkey, player_id: NonZeroU32) {
        self.game = Some(GameInfo {
            player_id,
            game_key,
        });
    }

    pub unsafe fn leave_game(&mut self) {
        self.game = None;
    }
}
