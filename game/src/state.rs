use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::bundled::{Bundle, Bundled};
use crate::error::Error;

pub const CURRENT_GAME_STATE_VERSION: u32 = 1;

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

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    log: Vec<Event>,
    state_step: u32,
    game_info: Pubkey,
}

impl State {
    fn new(key: Pubkey) -> Self {
        Self {
            log: vec![],
            state_step: 0,
            game_info: key,
        }
    }
}

impl<'s, 't0> Bundled<'s, 't0, State> {
    pub unsafe fn add_events(&mut self, state_step: u32, events: &[Event]) -> Result<(), Error> {
        if state_step == self.data().state_step {
            let state = self.data_mut();
            state.state_step += events.len() as u32;
            state.log.extend_from_slice(events);
            Ok(())
        } else {
            Err(Error::WrongStateStep)
        }
    }
    pub fn game_key(&self) -> Pubkey {
        self.data().game_info
    }
}

type InitializationArgs = Pubkey; // game_info address

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, InitializationArgs> for State {
    type Error = Error;

    fn new(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        let game_info = initialization_args;
        let game_state = next_account_info(accounts_iter)?;

        let data: &[u8] = &game_state.data.borrow();
        let mut buf = &*data;

        //Check previous versions
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(1) => {
                State::deserialize(&mut buf)
                    //Error occurs if account was already initialized
                    .map_or(Ok(()), |_| Err(Error::AlreadyCreated))?;
            }
            Ok(_) => {
                return Err(Error::WrongAccountVersion);
            }
            _ => {}
        }

        let state = State::new(game_info);
        Ok(unsafe { Bundled::new(state, game_state) })
    }
    fn unpack(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error> {
        let game_state = next_account_info(accounts_iter)?;

        if game_state.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        let mut data: &[u8] = &game_state.data.borrow();
        //Check previous versions
        let version = <u32>::deserialize(&mut data);
        let state = match version {
            Ok(0) => Err(Error::EmptyAccount),
            Ok(1) => State::deserialize(&mut data).map_err(|_| Error::CorruptedAccount),
            Ok(_) => Err(Error::WrongAccountVersion),
            _ => Err(Error::CorruptedAccount),
        }?;

        Ok(unsafe { Bundled::new(state, game_state) })
    }
    fn pack(bundle: Bundled<'s, 't0, Self>) -> Result<(), Self::Error> {
        let (state, account) = unsafe { bundle.release() };

        let mut data: &mut [u8] = &mut account.data.borrow_mut();
        (CURRENT_GAME_STATE_VERSION, state)
            .serialize(&mut data)
            .map_err(|e| Error::ProgramError(ProgramError::from(e)))
    }
}
