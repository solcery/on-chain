use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

pub const CURRENT_GAME_STATE_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    log: Vec<Event>,
    state_step: u32,
    game_info: Pubkey,
}

impl State {
    pub unsafe fn init(key: Pubkey) -> Self {
        Self {
            log: vec![],
            state_step: 0,
            game_info: key,
        }
    }

    pub unsafe fn from_raw_parts(log: Vec<Event>, state_step: u32, game_info: Pubkey) -> Self {
        Self {
            log,
            state_step,
            game_info,
        }
    }

    pub unsafe fn add_events(
        &mut self,
        state_step: u32,
        events: &[Event],
    ) -> Result<(), WrongStateStep> {
        if state_step == self.state_step {
            self.state_step += events.len() as u32;
            self.log.extend_from_slice(events);
            Ok(())
        } else {
            Err(WrongStateStep {})
        }
    }

    pub fn game_key(&self) -> Pubkey {
        self.game_info
    }
}
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
pub struct WrongStateStep {}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
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
