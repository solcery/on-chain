use crate::error::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::AccountInfo;
use solana_program::pubkey::Pubkey;
use std::num::NonZeroU32;

pub const CURRENT_PLAYER_VERSION: u32 = 1;

#[derive(Clone, Debug)]
pub struct Player<'a> {
    account: &'a AccountInfo<'a>,
    player_data: Data,
}
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
//TODO: Add correct Ord implementation
pub struct Data {
    pubkey: Pubkey,
    items: Vec<(u32, Pubkey)>,
    player_id: Option<NonZeroU32>,
}

impl<'a> Player<'a> {
    #[must_use]
    pub fn new(
        program_id: &'a Pubkey,
        signer: &'a AccountInfo<'a>,
        player_info: &'a AccountInfo<'a>,
    ) -> Result<Self, Error> {
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
                Data::deserialize(&mut buf)
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
            Ok(Self {
                account: player_info,
                player_data: Data::from_pubkey(pda),
            })
        } else {
            Err(Error::WrongPlayerAccount)
        }
    }

    #[must_use]
    pub fn unpack(
        program_id: &Pubkey,
        signer: &AccountInfo,
        player_info: &'a AccountInfo<'a>,
    ) -> Result<Self, Error> {
        let (pda, _bump_seed) =
            Pubkey::find_program_address(&[b"player", signer.key.as_ref()], program_id);

        if player_info.owner != program_id {
            return Err(Error::WrongPlayerAccount);
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
            Ok(1) => Data::deserialize(&mut data).map_err(|_| Error::CorruptedAccount),
            Ok(_) => Err(Error::WrongAccountVersion),
            _ => Err(Error::CorruptedAccount),
        }?;
        if player_data.key() == signer.key {
            Ok(Self {
                account: player_info,
                player_data: player_data,
            })
        } else {
            Err(Error::WrongPlayerAccount)
        }
    }

    #[must_use]
    pub fn pack(self) -> Result<(), Error> {
        let mut data: &mut [u8] = &mut self.account.data.borrow_mut();
        (CURRENT_PLAYER_VERSION, self.player_data)
            .serialize(&mut data)
            .map_err(|_| Error::AccountTooSmall)
    }
}

impl<'a> PartialEq for Player<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.player_data == other.player_data
    }
}

impl Data {
    #[must_use]
    pub fn from_pubkey(pubkey: Pubkey) -> Self {
        Self {
            pubkey,
            items: vec![],
            player_id: None,
        }
    }

    #[must_use]
    pub fn key(&self) -> &Pubkey {
        &self.pubkey
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
