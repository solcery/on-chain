use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::container::Container;
use crate::error::Error;
use crate::player::{Data as PlayerData, Player};

pub const CURRENT_GAME_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct Game<'a> {
    account: &'a AccountInfo<'a>,
    game_data: Data,
}

impl<'a> Game<'a> {
    #[must_use]
    pub fn new(
        program_id: &'a Pubkey,
        signer: &'a AccountInfo<'a>,
        player_info: &'a AccountInfo<'a>,
        project: &'a AccountInfo<'a>,
        game_info: &'a AccountInfo<'a>,
        num_players: u32,
        max_items: u32,
    ) -> Result<Self, Error> {
        Player::unpack(program_id, signer, player_info)?;

        if !game_info.is_writable {
            return Err(Error::NotWritable);
        }

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
                Data::deserialize(&mut buf)
                    //Error occurs if account was already initialized
                    .map_or(Ok(()), |_| Err(Error::AlreadyCreated))?;
            }
            Ok(_) => {
                return Err(Error::WrongAccountVersion);
            }
            _ => {}
        }

        let game = unsafe {
            Data::from_raw_parts(
                *project.key,
                Status::Initialization {
                    remaining_players: num_players,
                },
                *game_info.key,
                0,
                vec![],
            )
        };

        Ok(Self {
            account: game_info,
            game_data: game,
        })
    }

    #[must_use]
    pub fn unpack(
        program_id: &'a Pubkey,
        signer: &'a AccountInfo<'a>,
        player_info: &'a AccountInfo<'a>,
        game_info: &'a AccountInfo<'a>,
    ) -> Result<Self, Error> {
        unimplemented!();
    }

    #[must_use]
    pub fn pack(self) -> Result<(), Error> {
        if !self.account.is_writable {
            return Err(Error::NotWritable);
        }

        let mut data: &mut [u8] = &mut self.account.data.borrow_mut();
        (CURRENT_GAME_VERSION, self.game_data)
            .serialize(&mut data)
            .map_err(|_| Error::AccountTooSmall)
    }
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Data {
    project: Pubkey,
    status: Status,
    state: Pubkey,
    state_step: u32,
    players: Vec<PlayerData>,
}

impl Data {
    pub unsafe fn from_raw_parts(
        project: Pubkey,
        status: Status,
        state: Pubkey,
        state_step: u32,
        players: Vec<PlayerData>,
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
