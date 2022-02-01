use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::bundled::{Bundle, Bundled};
use crate::container::Container;
use crate::error::Error;
use crate::player::Player;

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

type InitializationArgs = (u32, u32); // num_players and max_items

impl<'a> Bundle<'a, InitializationArgs> for Game {
    type Error = Error;

    fn new(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'a, Self>, Self::Error> {
        let (num_players, max_items) = initialization_args;

        Player::unpack(program_id, accounts_iter)?;

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

        let game = unsafe {
            Game::from_raw_parts(
                *project.key,
                Status::Initialization {
                    remaining_players: num_players,
                },
                *game_info.key,
                0,
                vec![],
            )
        };

        Ok(unsafe { Bundled::new(game, vec![game_info]) })
    }
    fn unpack(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
    ) -> Result<Bundled<'a, Self>, Self::Error> {
        unimplemented!();
    }
    fn pack(bundle: Bundled<'a, Self>) -> Result<(), Self::Error> {
        let (game_data, accounts) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut accounts[0].data.borrow_mut();
        (CURRENT_GAME_VERSION, game_data)
            .serialize(&mut data)
            .map_err(|_| Error::AccountTooSmall)
    }
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
