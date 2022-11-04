use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub use super::{
    bundled::{Bundle, Bundled},
    game::Game,
    player::Player,
};
use crate::error::Error;

pub const CURRENT_GAME_STATE_VERSION: u32 = 1;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize)]
pub struct State {
    log: Vec<Event>,
    state_step: u32,
    game_info: Pubkey,
}

impl State {
    #[must_use]
    pub unsafe fn init(key: Pubkey) -> Self {
        Self {
            log: vec![],
            state_step: 0,
            game_info: key,
        }
    }

    #[must_use]
    pub unsafe fn from_raw_parts(log: Vec<Event>, state_step: u32, game_info: Pubkey) -> Self {
        Self {
            log,
            state_step,
            game_info,
        }
    }

    pub unsafe fn add_events(
        &mut self,
        state_step: u32,
        events: &[Event],
    ) -> Result<(), WrongStateStep> {
        if state_step == self.state_step {
            self.state_step += events.len() as u32;
            self.log.extend_from_slice(events);
            Ok(())
        } else {
            Err(WrongStateStep {})
        }
    }

    #[must_use]
    pub fn game_key(&self) -> Pubkey {
        self.game_info
    }
}
#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
pub struct WrongStateStep {}

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, BorshSerialize, BorshDeserialize,
)]
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

impl<'s, 't0> Bundled<'s, 't0, State> {
    pub fn add_events<'a, 'b>(
        &mut self,
        player: &Bundled<'a, 'b, Player>,
        game: &Bundled<'a, 'b, Game>,
        state_step: u32,
        events: &[Event],
    ) -> Result<(), Error> {
        if player.data().game_key() != Some(game.key()) {
            return Err(Error::NotInGame);
        }

        if self.data().game_key() != game.key() {
            return Err(Error::StateAccountMismatch);
        }

        debug_assert_eq!(self.key(), game.state_key());

        unsafe {
            // SAFETY: It was checked, that state, game and player are consistent.
            self.data_mut()
                .add_events(state_step, events)
                .map_err(|_| Error::WrongStateStep)
        }
    }
}

// FIXME: Actually, this is unsafe, because we rely on the assumption, that this is indeed game_info
// pubkey
type InitializationArgs = Pubkey; // game_info address

impl<'r, 's, 't0, 't1> Bundle<'r, 's, 't0, 't1, InitializationArgs> for State {
    type Error = Error;

    fn new<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
        initialization_args: InitializationArgs,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
        let game_info = initialization_args;
        let game_state = next_account_info(accounts_iter)?;

        if game_state.owner != program_id {
            return Err(Error::WrongAccountOwner);
        }

        let data: &[u8] = &game_state.data.borrow();
        let mut buf = data;

        //Check previous versions
        let version = <u32>::deserialize(&mut buf);
        match version {
            Ok(0) => {} // Default value
            Ok(_) => {
                return Err(Error::AlreadyInUse);
            }
            _ => {}
        }

        let state = unsafe { State::init(game_info) };
        Ok(unsafe { Bundled::new(state, game_state) })
    }

    fn unpack<AccountIter>(
        program_id: &'r Pubkey,
        accounts_iter: &mut AccountIter,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>
    where
        AccountIter: Iterator<Item = &'s AccountInfo<'t0>>,
    {
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
