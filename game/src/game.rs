use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::num::NonZeroU32;

use crate::bundled::{Bundle, Bundled};
use crate::error::Error;
use crate::player::Player as PlayerInfo;

pub const CURRENT_GAME_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Game {
    project: Pubkey,
    status: Status,
    state: Pubkey,
    state_step: u32,
    players: Vec<Player>,
}

impl Game {
    pub unsafe fn init(project: Pubkey, state: Pubkey, num_players: u32) -> Self {
        Self {
            project,
            status: Status::Initialization {
                remaining_players: num_players,
            },
            state,
            state_step: 0,
            players: vec![],
        }
    }

    pub unsafe fn from_raw_parts(
        project: Pubkey,
        status: Status,
        state: Pubkey,
        state_step: u32,
        players: Vec<Player>,
    ) -> Self {
        Self {
            project,
            status,
            state,
            state_step,
            players,
        }
    }
}

impl<'a> Bundled<'a, Game> {
    pub fn add_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        if player.in_game() {
            return Err(Error::AlreadyInGame);
        }

        let game_key = self.key();
        let game: &mut Game = self.data_mut();
        match game.status {
            Status::Initialization { remaining_players } => {
                if remaining_players > 0 {
                    // SAFETY: .len() + 1 is guaranteed to be greater than zero
                    let id = unsafe { NonZeroU32::new_unchecked(game.players.len() as u32 + 1) };
                    let player_key = player.key();
                    unsafe {
                        //SAFETY: game and player are changed synchronously, so the invariants are preserved
                        player.set_game(game_key, id);
                    }
                    game.players.push(Player {
                        key: player_key,
                        id,
                    });
                    Ok(())
                } else {
                    Err(Error::NoPlayerSlots)
                }
            }
            _ => Err(Error::GameStarted),
        }
    }
    pub fn remove_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        let game: &mut Game = self.data_mut();
        match &game.status {
            Status::Finished { winners } => {
                let player_key = player.key();
                let player_index = game.players.iter().position(|x| x.key == player_key);

                if let Some(index) = player_index {
                    game.players.swap_remove(index);

                    // Just to be completely paranoid.
                    // This assert should never fail.
                    debug_assert_eq!(player.game_key(), Some(self.key()));

                    unsafe {
                        //SAFETY: game and player are changed synchronously, so the invariants are preserved
                        player.leave_game()
                    };
                    Ok(())
                } else {
                    Err(Error::NotInGame)
                }
            }
            _ => Err(Error::NotFinished),
        }
    }
}

type InitializationArgs = (u32, u32); // num_players and max_items

impl<'a> Bundle<'a, InitializationArgs> for Game {
    type Error = Error;

    fn new(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'a, Self>, Self::Error> {
        // How to use max_items?
        let (num_players, max_items) = initialization_args;

        //Do we really need a player account for game creation?
        PlayerInfo::unpack(program_id, accounts_iter)?;

        let project = next_account_info(accounts_iter)?;
        let game_info = next_account_info(accounts_iter)?;

        let project_data: &[u8] = &project.data.borrow();
        let mut project_buf = &*project_data;

        Project::deserialize(&mut project_buf).map_err(|_| Error::WrongProjectAccount)?;

        let data: &[u8] = &game_info.data.borrow();
        let mut buf = &*data;

        //Check previous versions
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(1) => {
                Game::deserialize(&mut buf)
                    //Error occurs if account was already initialized
                    .map_or(Ok(()), |_| Err(Error::AlreadyCreated))?;
            }
            Ok(_) => {
                return Err(Error::WrongAccountVersion);
            }
            _ => {}
        }

        let game = unsafe { Game::init(*project.key, *game_info.key, num_players) };

        Ok(unsafe { Bundled::new(game, game_info) })
    }
    fn unpack(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
    ) -> Result<Bundled<'a, Self>, Self::Error> {
        unimplemented!();
    }
    fn pack(bundle: Bundled<'a, Self>) -> Result<(), Self::Error> {
        let (game_data, account) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut account.data.borrow_mut();
        (CURRENT_GAME_VERSION, game_data)
            .serialize(&mut data)
            .map_err(|_| Error::AccountTooSmall)
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Player {
    id: NonZeroU32,
    key: Pubkey,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Project {
    //By now it is empty, as we can't validate anything.
    //Later we'll add needed information
    //
    //This is a possible layout:
    //instructions: Pubkey,
    //object_types: Pubkey,
    min_players: u32,
    max_players: u32,
    //symtab: SymbolTable,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub enum Status {
    Initialization { remaining_players: u32 },
    Canceled,
    Started,
    Finished { winners: Vec<Pubkey> },
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Object {
    id: u32,
    tpl_id: u32,
    attrs: Vec<u32>,
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    objects: Vec<Object>,
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
