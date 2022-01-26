use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use super::Player;
use crate::container::{Container, Extractable};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Game {
    pub game_project: Pubkey,
    pub state_pubkey: Pubkey,
    pub state_step: u32,
    pub players: Vec<Player>,
    pub finished: bool,
    pub winners: Vec<Pubkey>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct GameObject {
    pub id: u32,
    pub tpl_id: u32,
    pub attrs: Vec<u32>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct GameState {
    objects: Vec<GameObject>,
}

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

impl Extractable for Event {
    fn extract(
        containered_data: Container<Event>,
        accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>>,
    ) -> Result<Event, ProgramError> {
        match containered_data {
            Container::InPlace(event) => Ok(event),
            Container::InAccount(pubkey) => {
                let event_account = next_account_info(accounts_iter)?;
                if *event_account.key == pubkey {
                    let event = Event::deserialize(&mut event_account.data.borrow().as_ref())?;
                    Ok(event)
                } else {
                    //TODO: We need more descriptive error here
                    Err(ProgramError::InvalidAccountData)
                }
            }
        }
    }
}
