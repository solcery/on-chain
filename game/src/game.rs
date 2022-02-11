use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};
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
    unsafe fn init(project: Pubkey, state: Pubkey, num_players: u32, max_items: u32) -> Self {
        Self {
            project,
            status: Status::Initialization {
                remaining_players: num_players,
                max_items,
            },
            state,
            state_step: 0,
            players: vec![],
        }
    }

    unsafe fn from_raw_parts(
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

    fn item_count(&self) -> usize {
        self.players
            .iter()
            .fold(0, |acc, player| acc + player.items.len())
    }
}

impl<'s, 't0> Bundled<'s, 't0, Game> {
    pub fn add_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        if player.in_game() {
            return Err(Error::AlreadyInGame);
        }

        let game_key = self.key();
        let game: &mut Game = self.data_mut();
        match game.status {
            Status::Initialization {
                remaining_players, ..
            } => {
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
    pub fn remove_player(&mut self, player: &mut PlayerInfo) -> Result<(), Error> {
        let game: &mut Game = self.data_mut();
        match &game.status {
            //TODO: State::Canceled is not used as by now we do not have CancelGame instruction
            Status::Finished { winners: _ } => {
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
    pub fn set_status(&mut self, new_status: Status) -> Result<(), Error> {
        let game: &mut Game = self.data_mut();
        match (&game.status, new_status) {
            (Status::Initialization { .. }, Status::Canceled) => {
                game.status = Status::Canceled;
                Ok(())
            }
            (
                Status::Initialization {
                    remaining_players, ..
                },
                Status::Started,
            ) => {
                if *remaining_players == 0 {
                    game.status = Status::Started;
                    Ok(())
                } else {
                    Err(Error::NotAllPlayersReady)
                }
            }
            (Status::Started, Status::Finished { winners }) => {
                game.status = Status::Finished { winners };
                Ok(())
            }
            _ => Err(Error::IllegalStatusChange),
        }
    }

    pub fn add_items(
        &mut self,
        player: &PlayerInfo,
        items: Vec<(&AccountInfo, &AccountInfo)>,
    ) -> Result<(), Error> {
        let game = self.data_mut();

        let player_key = player.key();
        let player_index = game
            .players
            .iter()
            .position(|x| x.key == player_key)
            .ok_or(Error::NotInGame)?;

        if let Status::Initialization { max_items, .. } = &game.status {
            if items.len() > *max_items as usize {
                return Err(Error::TooManyItems);
            }

            // It is required, that each item in the game has unique id.
            // This ids are NonZeroU32 derived from the number of already added items, so that
            // the first added item will have id=1, second - id=2 and so on.
            let mut item_id = unsafe {
                // SAFETY: always item_count returns value >= 0 and there will be definitely less
                // items than u32::MAX
                NonZeroU32::new_unchecked(game.item_count() as u32 + 1)
            };

            for item_bundle in items.iter() {
                let (token, mint) = item_bundle;

                let token_account = Account::unpack_from_slice(&token.data.borrow())?;
                let mint_key = token_account.mint;

                if mint_key != *mint.key {
                    return Err(Error::WrongAccountMint);
                }

                if token_account.owner != player.key() {
                    return Err(Error::NotOwnedNFT);
                }

                let mint = Mint::unpack_from_slice(&mint.data.borrow())?;

                if mint.mint_authority.is_some() {
                    return Err(Error::NotAnNFT);
                }

                if mint.supply != 1 {
                    return Err(Error::NotAnNFT);
                }

                if mint.decimals != 0 {
                    // IMO, this is unnecessary -- we've already checked that supply == 1.
                    return Err(Error::NotAnNFT);
                }

                // So, now this token is definitely an NFT

                // Check, that the player has not already added this NFT
                let player_info = &mut game.players[player_index];

                for item in player_info.items.iter() {
                    // Here we check only in the player's items, because we require, that the token
                    // is owned by that player.

                    // EXPLOIT: Player1 add item, transfer ownership to Player2, than Player2 is
                    // able to add the same item.

                    // FIXME: We should implemet a function, that check NFTs against all the items
                    // in the game
                    if &item.token == token.key {
                        return Err(Error::TokenAlreadyInGame);
                    }
                }

                let new_item = Item {
                    id: item_id,
                    token: *token.key,
                };

                player_info.items.push(new_item);
                item_id = unsafe { NonZeroU32::new_unchecked(u32::from(item_id) + 1) };
            }
            Ok(())
        } else {
            Err(Error::GameStarted)
        }
    }
}

type InitializationArgs = (u32, u32); // num_players and max_items

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, InitializationArgs> for Game {
    type Error = Error;

    fn new(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        // How to use max_items?
        let (num_players, max_items) = initialization_args;

        //Do we really need a player account for game creation?
        PlayerInfo::unpack(program_id, accounts_iter)?;

        let project = next_account_info(accounts_iter)?;
        let game_info = next_account_info(accounts_iter)?;
        let game_state = next_account_info(accounts_iter)?;

        let project_data: &[u8] = &project.data.borrow();
        let mut project_buf = &*project_data;

        let project_struct =
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
                let mut state_data: &[u8] = &game_info.data.borrow();
                Game::deserialize(&mut state_data)
                    //Error occurs if account was already initialized
                    .map_or(Ok(()), |_| Err(Error::AlreadyCreated))?;
            }
            Ok(_) => {
                return Err(Error::WrongAccountVersion);
            }
            _ => {}
        }

        let players_range = project_struct.min_players..=project_struct.max_players;
        if players_range.contains(&num_players) {
            let game = unsafe { Game::init(*project.key, *game_state.key, num_players, max_items) };
            Ok(unsafe { Bundled::new(game, game_info) })
        } else {
            Err(Error::WrongPlayerNumber)
        }
    }
    fn unpack(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        // FIXME: now unpack() method itself does not check anyting about signer.
        // Maybe, it is ok, since all the interractions with Game require Player account, which
        // perform checks.
        // Maybe we should add another check here. Smth like "check that the signer has a player
        // account and it is participating in the game (this is not correct, as it will break
        // join_game)"
        let game_info = next_account_info(accounts_iter)?;

        if game_info.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        let mut data: &[u8] = &game_info.data.borrow();
        //Check previous versions
        let version = <u32>::deserialize(&mut data);
        let game_data = match version {
            Ok(0) => Err(Error::EmptyAccount),
            Ok(1) => Game::deserialize(&mut data).map_err(|_| Error::CorruptedAccount),
            Ok(_) => Err(Error::WrongAccountVersion),
            _ => Err(Error::CorruptedAccount),
        }?;

        Ok(unsafe { Bundled::new(game_data, game_info) })
    }
    fn pack(bundle: Bundled<'s, 't0, Self>) -> Result<(), Self::Error> {
        let (game_data, account) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut account.data.borrow_mut();
        (CURRENT_GAME_VERSION, game_data)
            .serialize(&mut data)
            .map_err(|e| Error::ProgramError(ProgramError::from(e)))
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
