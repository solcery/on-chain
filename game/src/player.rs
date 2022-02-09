use crate::bundled::{Bundle, Bundled};
use crate::error::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;

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

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, ()> for Player {
    type Error = Error;

    fn new(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
        _initialization_args: (),
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        let signer = next_account_info(accounts_iter)?;
        let player_info = next_account_info(accounts_iter)?;
        //player_info address check
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[b"player", signer.key.as_ref()], program_id);

        let data: &[u8] = &player_info.data.borrow();
        let mut buf = &*data;

        // Check previous versions
        // We need this check to prove, that the player does not try to wipe the existing account
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(1) => {
                Player::deserialize(&mut buf)
                    //Here error occurs if player account was already initialized
                    .map_or(Ok(()), |_| Err(Error::AlreadyCreated))?;
            }
            Ok(_) => {
                return Err(Error::WrongAccountVersion);
            }
            _ => {}
        }

        if !signer.is_signer {
            return Err(Error::NotSigned);
        }

        if *player_info.key == pda {
            Ok(unsafe { Bundled::new(Player::from_pubkey(pda), player_info) })
        } else {
            Err(Error::WrongPlayerAccount)
        }
    }
    fn unpack(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        let signer = next_account_info(accounts_iter)?;
        let player_info = next_account_info(accounts_iter)?;

        //player_info address check
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[b"player", signer.key.as_ref()], program_id);

        if player_info.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        //player_info address check
        if *player_info.key != pda {
            return Err(Error::WrongPlayerAccount);
        }

        if !signer.is_signer {
            return Err(Error::NotSigned);
        }

        let mut data: &[u8] = &player_info.data.borrow();
        //Check previous versions
        let version = <u32>::deserialize(&mut data);
        let player_data = match version {
            Ok(0) => Err(Error::EmptyAccount),
            Ok(1) => Player::deserialize(&mut data).map_err(|_| Error::CorruptedAccount),
            Ok(_) => Err(Error::WrongAccountVersion),
            _ => Err(Error::CorruptedAccount),
        }?;
        if player_data.key() == *signer.key {
            Ok(unsafe { Bundled::new(player_data, player_info) })
        } else {
            Err(Error::WrongPlayerAccount)
        }
    }
    fn pack(bundle: Bundled<'s, 't0, Self>) -> Result<(), Self::Error> {
        let (player_data, account) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut account.data.borrow_mut();
        (CURRENT_PLAYER_VERSION, player_data)
            .serialize(&mut data)
            .map_err(|_| Error::AccountTooSmall)
    }
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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    //TODO: move to SolceryPlayer protocol
    pub pubkey: Pubkey,
    pub game: Option<Pubkey>,
}

#[cfg(test)]
mod tests;
