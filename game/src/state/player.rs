use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, program_error::ProgramError,
};
use std::num::NonZeroU32;

use crate::error::Error;
use crate::state::bundled::{Bundle, Bundled};

pub const CURRENT_PLAYER_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
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
    pub unsafe fn from_raw_parts(
        pubkey: Pubkey,
        items: Vec<(u32, Pubkey)>,
        game: Option<GameInfo>,
    ) -> Self {
        Self {
            pubkey,
            items,
            game,
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

    /// # Safety
    ///
    /// Game and player structs must be changed syncroniusly
    pub unsafe fn set_game(&mut self, game_key: Pubkey, player_id: NonZeroU32) {
        self.game = Some(GameInfo {
            player_id,
            game_key,
        });
    }

    /// # Safety
    ///
    /// Game and player structs must be changed syncroniusly
    pub unsafe fn leave_game(&mut self) {
        self.game = None;
    }
}

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, ()> for Player {
    type Error = Error;

    fn new<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
        _initialization_args: (),
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
        let signer = next_account_info(accounts_iter)?;
        let player_info = next_account_info(accounts_iter)?;
        //player_info address check
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[b"player", signer.key.as_ref()], program_id);

        if player_info.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        let data: &[u8] = &player_info.data.borrow();
        let mut buf = data;

        // Check previous versions
        // We need this check to prove, that the player does not try to wipe the existing account
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(_) => {
                return Err(Error::AlreadyInUse);
            }
            _ => {}
        }

        if !signer.is_signer {
            return Err(Error::NotSigned);
        }

        if *player_info.key == pda {
            Ok(unsafe { Bundled::new(Player::from_pubkey(*signer.key), player_info) })
        } else {
            Err(Error::WrongPlayerAccount)
        }
    }
    fn unpack<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
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
            .map_err(|e| Error::ProgramError(ProgramError::from(e)))
    }
}

#[cfg(test)]
mod new;
