use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
};
use spl_token::state::{Account, Mint};

use crate::{
    bundled::{Bundle, Bundled},
    error::Error,
    player::Player as PlayerInfo,
};
pub use solcery_data_types::game::{Game, Status};
use solcery_data_types::game::{Project, CURRENT_GAME_PROJECT_VERSION, CURRENT_GAME_VERSION};
use solcery_data_types::state::State;

impl<'s, 't0> Bundled<'s, 't0, Game> {
    pub fn add_player<'a, 'b>(
        &mut self,
        player_bundle: &mut Bundled<'a, 'b, PlayerInfo>,
    ) -> Result<(), Error> {
        let player = player_bundle.data_mut();

        if player.in_game() {
            return Err(Error::AlreadyInGame);
        }

        let game_key = self.key();
        let game = self.data_mut();
        unsafe { game.add_player(game_key, player) }?;
        Ok(())
    }

    pub fn remove_player<'a, 'b>(
        &mut self,
        player_bundle: &mut Bundled<'a, 'b, PlayerInfo>,
    ) -> Result<(), Error> {
        let player = player_bundle.data_mut();
        let game = self.data_mut();
        game.remove_player(player)?;
        Ok(())
    }

    pub fn set_status<'a, 'b>(
        &mut self,
        player: &Bundled<'a, 'b, PlayerInfo>,
        new_status: Status,
    ) -> Result<(), Error> {
        if player.data().game_key() != Some(self.key()) {
            return Err(Error::NotInGame);
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
    ) -> Result<(), Error> {
        let player = player_bundle.data();
        if player.game_key() != Some(self.key()) {
            return Err(Error::NotInGame);
        }

        let game = self.data_mut();

        let items: Vec<_> = items
            .iter()
            .map(|(token, mint)| {
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
    type Error = Error;

    fn new<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
        // How to use max_items?
        let (num_players, max_items) = initialization_args;

        let project = next_account_info(accounts_iter)?;
        let game_info = next_account_info(accounts_iter)?;
        let game_state = next_account_info(accounts_iter)?;

        let project_data: &[u8] = &project.data.borrow();
        let mut project_buf = &*project_data;

        let (project_ver, project_struct) = <(u32, Project)>::deserialize(&mut project_buf)
            .map_err(|_| Error::WrongProjectAccount)?;

        if project_ver == 0 {
            return Err(Error::WrongProjectAccount);
        }

        if project.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        if project_ver != CURRENT_GAME_PROJECT_VERSION {
            return Err(Error::WrongProjectVersion);
        }

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
                State::deserialize(&mut state_data)
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

#[cfg(test)]
mod tests;
