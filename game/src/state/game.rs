use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};
use std::num::NonZeroU32;
use thiserror::Error;

use crate::error::Error as CrateError;
use crate::state::bundled::{Bundle, Bundled};
use crate::state::player::Player as PlayerInfo;

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
    #[must_use]
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

    #[must_use]
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

    #[must_use]
    pub fn item_count(&self) -> usize {
        self.players
            .iter()
            .fold(0, |acc, player| acc + player.items.len())
    }

    /// # Safety
    ///
    /// `game_key` must be the Pubkey of an Account, which stores `Self`
    pub unsafe fn add_player(
        &mut self,
        game_key: Pubkey,
        player: &mut PlayerInfo,
    ) -> Result<(), Error> {
        match &mut self.status {
            Status::Initialization {
                remaining_players, ..
            } => {
                if *remaining_players > 0 {
                    // SAFETY: .len() + 1 is guaranteed to be greater than zero and less than
                    // U32::MAX
                    let id = NonZeroU32::new_unchecked(self.players.len() as u32 + 1);
                    let player_key = player.key();
                    //SAFETY: game and player are changed synchronously, so the invariants are preserved
                    player.set_game(game_key, id);
                    self.players.push(Player {
                        key: player_key,
                        id,
                        items: vec![],
                    });
                    *remaining_players -= 1;
                    Ok(())
                } else {
                    Err(Error::NoPlayerSlots)
                }
            }
            _ => Err(Error::GameStarted),
        }
    }

    pub fn remove_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        match &self.status {
            Status::Finished { winners: _ } | Status::Canceled => {
                let player_key = player.key();
                let player_index = self.players.iter().position(|x| x.key == player_key);

                if let Some(index) = player_index {
                    self.players.swap_remove(index);

                    unsafe {
                        //SAFETY: game and player are changed synchronously, so the invariants are preserved
                        player.leave_game();
                    }
                    Ok(())
                } else {
                    Err(Error::NotInGame)
                }
            }
            _ => Err(Error::NotFinished),
        }
    }

    /// # Safety
    ///
    /// Only players participating in `self` are allowed to change the status.
    pub unsafe fn set_status(&mut self, new_status: Status) -> Result<(), Error> {
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

    /// # Safety
    ///
    /// items must contain only keys of valid NFTs
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

            for token in items {
                if self.token_in_game(token) {
                    return Err(Error::TokenAlreadyInGame);
                }

                let new_item = Item {
                    id: item_id,
                    token: *token,
                };

                self.players[player_index].items.push(new_item);
                item_id = NonZeroU32::new_unchecked(u32::from(item_id) + 1);
            }
            Ok(())
        } else {
            Err(Error::GameStarted)
        }
    }

    #[must_use]
    pub fn state_key(&self) -> Pubkey {
        self.state
    }

    #[must_use]
    pub fn token_in_game(&self, token: &Pubkey) -> bool {
        for player_info in &self.players {
            for item in &player_info.items {
                if &item.token == token {
                    return true;
                }
            }
        }
        false
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Player {
    id: NonZeroU32,
    key: Pubkey,
    items: Vec<Item>,
}

impl Player {
    #[must_use]
    pub unsafe fn from_raw_parts(id: NonZeroU32, key: Pubkey, items: Vec<Item>) -> Self {
        Self { id, key, items }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct Item {
    id: NonZeroU32,
    token: Pubkey,
}

impl Item {
    #[must_use]
    pub unsafe fn from_raw_parts(id: NonZeroU32, token: Pubkey) -> Self {
        Self { id, token }
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

impl<'s, 't0> Bundled<'s, 't0, Game> {
    pub fn add_player<'a, 'b>(
        &mut self,
        player_bundle: &mut Bundled<'a, 'b, PlayerInfo>,
    ) -> Result<(), CrateError> {
        let player = player_bundle.data_mut();

        if player.in_game() {
            return Err(CrateError::AlreadyInGame);
        }

        let game_key = self.key();
        let game = self.data_mut();
        unsafe { game.add_player(game_key, player) }?;
        Ok(())
    }

    pub fn remove_player<'a, 'b>(
        &mut self,
        player_bundle: &mut Bundled<'a, 'b, PlayerInfo>,
    ) -> Result<(), CrateError> {
        let player = player_bundle.data_mut();
        let game = self.data_mut();
        game.remove_player(player)?;
        Ok(())
    }

    pub fn set_status<'a, 'b>(
        &mut self,
        player: &Bundled<'a, 'b, PlayerInfo>,
        new_status: Status,
    ) -> Result<(), CrateError> {
        if player.data().game_key() != Some(self.key()) {
            return Err(CrateError::NotInGame);
        }

        let game: &mut Game = self.data_mut();
        unsafe {
            //SAFETY: We've checked, that the player belongs to that game

            game.set_status(new_status)?;
        }
        Ok(())
    }

    pub fn add_items<'a, 'b>(
        &mut self,
        player_bundle: &Bundled<'a, 'b, PlayerInfo>,
        items: Vec<(&AccountInfo, &AccountInfo)>,
    ) -> Result<(), CrateError> {
        let player = player_bundle.data();
        if player.game_key() != Some(self.key()) {
            return Err(CrateError::NotInGame);
        }

        let game = self.data_mut();

        let items: Vec<_> = items
            .iter()
            .map(|(token, mint)| {
                let token_account = Account::unpack_from_slice(&token.data.borrow())?;
                let mint_key = token_account.mint;

                if mint_key != *mint.key {
                    return Err(CrateError::WrongAccountMint);
                }

                if token_account.owner != player.key() {
                    return Err(CrateError::NotOwnedNFT);
                }

                let mint = Mint::unpack_from_slice(&mint.data.borrow())?;

                if mint.mint_authority.is_some() {
                    return Err(CrateError::NotAnNFT);
                }

                if mint.supply != 1 {
                    return Err(CrateError::NotAnNFT);
                }

                if mint.decimals != 0 {
                    // IMO, this is unnecessary -- we've already checked that supply == 1.
                    return Err(CrateError::NotAnNFT);
                }

                // So, now this token is definitely an NFT
                Ok(token.key)
            })
            .collect::<Result<_, _>>()?;

        unsafe {
            game.add_items(player, items)?;
        }
        Ok(())
    }

    pub fn state_key(&self) -> Pubkey {
        self.data().state_key()
    }
}

type InitializationArgs = (u32, u32); // num_players and max_items

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, InitializationArgs> for Game {
    type Error = CrateError;

    fn new<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
        let (num_players, max_items) = initialization_args;

        let project = next_account_info(accounts_iter)?;
        let game_info = next_account_info(accounts_iter)?;
        let game_state = next_account_info(accounts_iter)?;

        if game_info.owner != program_id {
            return Err(Self::Error::WrongAccountOwner);
        }

        let project_data: &[u8] = &project.data.borrow();
        let mut project_buf = project_data;

        let (project_ver, project_struct) = <(u32, Project)>::deserialize(&mut project_buf)
            .map_err(|_| Self::Error::WrongProjectAccount)?;

        if project.owner != program_id {
            return Err(Self::Error::WrongAccountOwner);
        }

        if project_ver != CURRENT_GAME_PROJECT_VERSION {
            return Err(Self::Error::WrongProjectVersion);
        }

        let data: &[u8] = &game_info.data.borrow();
        let mut buf = data;

        //Check previous versions
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(_) => {
                return Err(Self::Error::AlreadyInUse);
            }
            _ => {}
        }

        let mut state_data: &[u8] = &game_state.data.borrow();

        let version = <u32>::deserialize(&mut state_data);
        match version {
            Ok(0) => {} // Default value
            Ok(_) => {
                return Err(Self::Error::AlreadyInUse);
            }
            _ => {}
        }

        let players_range = project_struct.min_players..=project_struct.max_players;
        if players_range.contains(&num_players) {
            let game = unsafe { Game::init(*project.key, *game_state.key, num_players, max_items) };
            Ok(unsafe { Bundled::new(game, game_info) })
        } else {
            Err(Self::Error::WrongPlayerNumber)
        }
    }

    fn unpack<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
        // Maybe we should add another check here. Smth like "check that the signer has a player
        // account and it is participating in the game (this is not correct, as it will break
        // join_game)"
        let game_info = next_account_info(accounts_iter)?;

        if game_info.owner != program_id {
            return Err(Self::Error::WrongAccountOwner);
        }

        let mut data: &[u8] = &game_info.data.borrow();
        //Check previous versions
        let version = <u32>::deserialize(&mut data);
        let game_data = match version {
            Ok(0) => Err(Self::Error::EmptyAccount),
            Ok(1) => Game::deserialize(&mut data).map_err(|_| Self::Error::CorruptedAccount),
            Ok(_) => Err(Self::Error::WrongAccountVersion),
            _ => Err(Self::Error::CorruptedAccount),
        }?;

        Ok(unsafe { Bundled::new(game_data, game_info) })
    }
    fn pack(bundle: Bundled<'s, 't0, Self>) -> Result<(), Self::Error> {
        let (game_data, account) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut account.data.borrow_mut();
        (CURRENT_GAME_VERSION, game_data)
            .serialize(&mut data)
            .map_err(|e| Self::Error::ProgramError(ProgramError::from(e)))
    }
}

#[cfg(test)]
mod add_items;

#[cfg(test)]
mod new;
