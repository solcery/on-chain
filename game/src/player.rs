use crate::bundled::{Bundle, Bundled};
use crate::error::Error;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::next_account_info, account_info::AccountInfo, program_error::ProgramError,
};

use solana_program::pubkey::Pubkey;

pub use solcery_data_types::player::Player;
use solcery_data_types::player::CURRENT_PLAYER_VERSION;

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
            Ok(unsafe { Bundled::new(Player::from_pubkey(*signer.key), player_info) })
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
            .map_err(|e| Error::ProgramError(ProgramError::from(e)))
    }
}

#[cfg(test)]
mod tests;
