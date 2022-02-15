use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};

use crate::bundled::{Bundle, Bundled};
use crate::error::Error;
pub use solcery_data_types::state::{Event, State, CURRENT_GAME_STATE_VERSION};

impl<'s, 't0> Bundled<'s, 't0, State> {
    pub unsafe fn add_events(&mut self, state_step: u32, events: &[Event]) -> Result<(), Error> {
        self.data_mut()
            .add_events(state_step, events)
            .map_err(|_| Error::WrongStateStep)
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

        let state = State::init(game_info);
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
