#![deny(missing_debug_implementations)]
//#![deny(missing_docs)]

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};
use solcery_data_types::container::Container;

mod bundled;
mod error;
mod game;
mod player;
mod state;

use bundled::Bundle;
use error::Error;
use game::{Game, Status as GameStatus};
use player::Player;
use state::{Event, State};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub enum Instruction {
    /// Fill a special [Player](Player) account for signer, where all the metainformation will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    CreatePlayerAccount,
    /// Updates [Player](Player) account from old version.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    UpdatePlayerAccount,
    /// Fill  [Game](Game) account for signer, where all the metainformation of the game will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 2. `[]` GameProject account
    /// 3. `[writable]` Game account
    /// 4. `[writable]` GameState account
    CreateGame { num_players: u32, max_items: u32 },
    /// Add (Player)[Player] to the existing (Game)[Game].
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    JoinGame,
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    /// for each NFT this accounts should be provided:
    /// 1. `[]` token account
    /// 2. `[]` mint account
    AddItems { num_items: u32 },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 3. `[writable]` Game account
    SetGameStatus { new_game_status: GameStatus },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[]` Player account with correct PDA
    /// 3. `[writable]` Game account
    /// 4. `[]` (Optional) Event account
    AddEvent {
        event_container: Container<Vec<Event>>,
        state_step: u32,
    },
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 3. `[writable]` Game account
    LeaveGame,
}

entrypoint!(process_instruction_bytes);
pub fn process_instruction_bytes(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = Instruction::deserialize(&mut buf)?;

    // TODO: We probaly need a special feature for debug printing
    if cfg!(debug_assertions) {
        dbg!(process_instruction(program_id, accounts, instruction)).map_err(ProgramError::from)
    } else {
        process_instruction(program_id, accounts, instruction).map_err(ProgramError::from)
    }
}

fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction: Instruction,
) -> Result<(), Error> {
    let accounts_iter = &mut accounts.iter();
    match instruction {
        Instruction::CreatePlayerAccount => {
            let player = Player::new(program_id, accounts_iter, ())?;
            Bundle::pack(player)?;
        }
        Instruction::UpdatePlayerAccount => {
            let player = Player::unpack(program_id, accounts_iter)?;
            Bundle::pack(player)?;
        }
        Instruction::CreateGame {
            num_players,
            max_items,
        } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            if player.data().in_game() {
                return Err(Error::AlreadyInGame);
            }
            let game = Game::new(program_id, accounts_iter, (num_players, max_items))?;

            //both game and state use the same accounts, so account_iter have to be "restarted"
            let accounts_iter = &mut accounts.iter().skip(4);

            let state = State::new(program_id, accounts_iter, game.key())?;
            Bundle::pack(state)?;
            Bundle::pack(game)?;
        }
        Instruction::JoinGame => {
            let mut player = Player::unpack(program_id, accounts_iter)?;
            let mut game = Game::unpack(program_id, accounts_iter)?;

            game.add_player(&mut player)?;

            Bundle::pack(player)?;
            Bundle::pack(game)?;
        }
        Instruction::AddItems { num_items } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            let mut game = Game::unpack(program_id, accounts_iter)?;

            let mut items = Vec::<(&AccountInfo, &AccountInfo)>::with_capacity(num_items as usize);
            for _ in 0..num_items {
                let item = next_account_info(accounts_iter)?;
                let mint = next_account_info(accounts_iter)?;
                items.push((item, mint));
            }

            game.add_items(&player, items)?;

            Bundle::pack(game)?;
        }
        Instruction::SetGameStatus { new_game_status } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            let mut game = Game::unpack(program_id, accounts_iter)?;

            game.set_status(&player, new_game_status)?;

            Bundle::pack(game)?;
        }
        Instruction::AddEvent {
            event_container,
            state_step,
        } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            let game = Game::unpack(program_id, accounts_iter)?;
            let mut state = State::unpack(program_id, accounts_iter)?;

            let events = Container::extract(event_container, accounts_iter)?;

            state.add_events(&player, &game, state_step, &events)?;

            Bundle::pack(state)?;
        }
        Instruction::LeaveGame => {
            let mut player = Player::unpack(program_id, accounts_iter)?;
            let mut game = Game::unpack(program_id, accounts_iter)?;

            game.remove_player(&mut player)?;

            Bundle::pack(player)?;
            Bundle::pack(game)?;
        }
    }
    Ok(())
}
