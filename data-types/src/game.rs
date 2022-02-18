use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use std::num::NonZeroU32;
use thiserror::Error;

use crate::player::Player as PlayerInfo;
pub const CURRENT_GAME_VERSION: u32 = 1;
pub const CURRENT_GAME_PROJECT_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Game {
    project: Pubkey,
    status: Status,
    state: Pubkey,
    players: Vec<Player>,
}

impl Game {
    pub unsafe fn init(project: Pubkey, state: Pubkey, num_players: u32, max_items: u32) -> Self {
        Self {
            project,
            status: Status::Initialization {
                remaining_players: num_players,
                max_items,
            },
            state,
            players: vec![],
        }
    }

    pub unsafe fn from_raw_parts(
        project: Pubkey,
        status: Status,
        state: Pubkey,
        players: Vec<Player>,
    ) -> Self {
        Self {
            project,
            status,
            state,
            players,
        }
    }

    pub fn item_count(&self) -> usize {
        self.players
            .iter()
            .fold(0, |acc, player| acc + player.items.len())
    }

    pub unsafe fn add_player(
        &mut self,
        game_key: Pubkey,
        player: &mut PlayerInfo,
    ) -> Result<(), Error> {
        match self.status {
            Status::Initialization {
                remaining_players, ..
            } => {
                if remaining_players > 0 {
                    // SAFETY: .len() + 1 is guaranteed to be greater than zero
                    let id = NonZeroU32::new_unchecked(self.players.len() as u32 + 1);
                    let player_key = player.key();
                    //SAFETY: game and player are changed synchronously, so the invariants are preserved
                    player.set_game(game_key, id);
                    self.players.push(Player {
                        key: player_key,
                        id,
                        items: vec![],
                    });
                    Ok(())
                } else {
                    Err(Error::NoPlayerSlots)
                }
            }
            _ => Err(Error::GameStarted),
        }
    }

    pub unsafe fn remove_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        match &self.status {
            //TODO: State::Canceled is not used as by now we do not have CancelGame instruction
            Status::Finished { winners: _ } => {
                let player_key = player.key();
                let player_index = self.players.iter().position(|x| x.key == player_key);

                if let Some(index) = player_index {
                    self.players.swap_remove(index);

                    //SAFETY: game and player are changed synchronously, so the invariants are preserved
                    player.leave_game();
                    Ok(())
                } else {
                    Err(Error::NotInGame)
                }
            }
            _ => Err(Error::NotFinished),
        }
    }

    pub fn set_status(&mut self, new_status: Status) -> Result<(), Error> {
        match (&self.status, new_status) {
            (Status::Initialization { .. }, Status::Canceled) => {
                self.status = Status::Canceled;
                Ok(())
            }
            (
                Status::Initialization {
                    remaining_players, ..
                },
                Status::Started,
            ) => {
                if *remaining_players == 0 {
                    self.status = Status::Started;
                    Ok(())
                } else {
                    Err(Error::NotAllPlayersReady)
                }
            }
            (Status::Started, Status::Finished { winners }) => {
                self.status = Status::Finished { winners };
                Ok(())
            }
            _ => Err(Error::IllegalStatusChange),
        }
    }

    pub unsafe fn add_items(
        &mut self,
        player: &PlayerInfo,
        items: Vec<&Pubkey>,
    ) -> Result<(), Error> {
        let player_key = player.key();
        let player_index = self
            .players
            .iter()
            .position(|x| x.key == player_key)
            .ok_or(Error::NotInGame)?;

        if let Status::Initialization { max_items, .. } = &self.status {
            if items.len() > *max_items as usize {
                return Err(Error::TooManyItems);
            }

            // It is required, that each item in the game has unique id.
            // This ids are NonZeroU32 derived from the number of already added items, so that
            // the first added item will have id=1, second - id=2 and so on.

            // SAFETY: item_count always returns value >= 0 and there will be definitely less
            // items than u32::MAX
            let mut item_id = NonZeroU32::new_unchecked(self.item_count() as u32 + 1);

            for token in items.into_iter() {
                // Check, that the player has not already added this NFT
                let player_info = &mut self.players[player_index];

                for item in player_info.items.iter() {
                    // Here we check only in the player's items, because we require, that the token
                    // is owned by that player.

                    // EXPLOIT: Player1 add item, transfer ownership to Player2, than Player2 is
                    // able to add the same item.

                    // FIXME: We should implemet a function, that check NFTs against all the items
                    // in the game
                    if &item.token == token {
                        return Err(Error::TokenAlreadyInGame);
                    }
                }

                let new_item = Item {
                    id: item_id,
                    token: *token,
                };

                player_info.items.push(new_item);
                item_id = NonZeroU32::new_unchecked(u32::from(item_id) + 1);
            }
            Ok(())
        } else {
            Err(Error::GameStarted)
        }
    }

    pub fn state_key(&self) -> Pubkey {
        self.state
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Player {
    id: NonZeroU32,
    key: Pubkey,
    items: Vec<Item>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Item {
    id: NonZeroU32,
    token: Pubkey,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Project {
    //By now it is empty, as we can't validate anything.
    //Later we'll add needed information
    //
    //This is a possible layout:
    //instructions: Pubkey,
    //object_types: Pubkey,
    pub min_players: u32,
    pub max_players: u32,
    //symtab: SymbolTable,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub enum Status {
    Initialization {
        remaining_players: u32,
        max_items: u32,
    },
    Canceled,
    Started,
    Finished {
        winners: Vec<Pubkey>,
    },
}

#[derive(
    Error, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
pub enum Error {
    #[error("The game has already started")]
    GameStarted,
    #[error("Illegal status change")]
    IllegalStatusChange,
    #[error("No player slots left")]
    NoPlayerSlots,
    #[error("Not all players have joined the game")]
    NotAllPlayersReady,
    #[error("The game is not finished")]
    NotFinished,
    #[error("Player not in this game")]
    NotInGame,
    #[error("The supplied token is already in game")]
    TokenAlreadyInGame,
    #[error("Attempted to add too many items")]
    TooManyItems,
}
