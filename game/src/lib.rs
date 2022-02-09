use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
};

mod bundled;
mod container;
mod error;
mod game;
mod player;

use bundled::Bundle;
use container::Container;
use error::Error;
use game::{Event, Game, Status as GameStatus};
use player::Player;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
//TODO: Add conversion tests
pub enum Instruction {
    /// Fills a special [Player](Player) account for signer, where all the metainformation will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    //TODO: we should probably provide a way to create this account
    /// 1. `[writable]` Player account with correct PDA
    CreatePlayerAccount,
    /// Updates [Player](Player) account from old version.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    UpdatePlayerAccount,
    /// Fills  [Game](Game) account for signer, where all the metainformation  of the game will be stored.
    ///
    /// Accounts expected:
    ///
    /// 0. `[signer]` The account of the person, who will be playing.
    /// 1. `[writable]` Player account with correct PDA
    /// 2. `[]` GameProject account
    //TODO: we should probably provide a way to create this account
    /// 3. `[writable]` Game account
    CreateGame {
        num_players: u32,
        max_items: u32,
    },
    JoinGame,
    AddItems {
        num_items: u32,
    },
    SetGameStatus {
        new_game_status: GameStatus,
    },
    AddEvent(Container<Event>),
    LeaveGame,
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let mut buf = instruction_data;
    let instruction = Instruction::deserialize(&mut buf)?;

    let accounts_iter = &mut accounts.iter();
    match instruction {
        Instruction::CreatePlayerAccount => {
            let player = Player::new(program_id, accounts_iter, ())?;
            Bundle::pack(player).map_err(ProgramError::from)
        }
        Instruction::UpdatePlayerAccount => {
            let player = Player::unpack(program_id, accounts_iter)?;
            Bundle::pack(player).map_err(ProgramError::from)
        }
        Instruction::CreateGame {
            num_players,
            max_items,
        } => {
            let game = Game::new(program_id, accounts_iter, (num_players, max_items))?;
            Bundle::pack(game).map_err(ProgramError::from)
        }
        Instruction::JoinGame => {
            let mut player = Player::unpack(program_id, accounts_iter)?;
            //FIXME: quick hack caused by the fact, that both player and game are using signer and
            //player_info accounts
            let accounts_iter = &mut accounts.iter();
            let mut game = Game::unpack(program_id, accounts_iter)?;
            game.add_player(player.data_mut())?;
            Bundle::pack(player)?;
            Bundle::pack(game).map_err(ProgramError::from)
        }
        Instruction::AddItems { num_items } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            //FIXME: quick hack caused by the fact, that both player and game are using signer and
            //player_info accounts
            let accounts_iter = &mut accounts.iter();
            let mut game = Game::unpack(program_id, accounts_iter)?;

            if player.data().game_key() == Some(game.key()) {
                let mut items =
                    Vec::<(&AccountInfo, &AccountInfo)>::with_capacity(num_items as usize);
                for _ in 0..num_items {
                    let item = next_account_info(accounts_iter)?;
                    let mint = next_account_info(accounts_iter)?;
                    items.push((item, mint));
                }
                game.add_items(player.data(), items)?;
                Bundle::pack(game).map_err(ProgramError::from)
            } else {
                Err(ProgramError::from(Error::NotInGame))
            }
        }
        Instruction::SetGameStatus { new_game_status } => {
            let player = Player::unpack(program_id, accounts_iter)?;
            //FIXME: quick hack caused by the fact, that both player and game are using signer and
            //player_info accounts
            let accounts_iter = &mut accounts.iter();
            let mut game = Game::unpack(program_id, accounts_iter)?;

            if player.data().game_key() == Some(game.key()) {
                game.set_status(new_game_status)?;
                Bundle::pack(game).map_err(ProgramError::from)
            } else {
                Err(ProgramError::from(Error::NotInGame))
            }
        }
        Instruction::AddEvent(event_container) => {
            //let signer = next_account_info(accounts_iter)?;
            //let player = next_account_info(accounts_iter)?;
            //let game = next_account_info(accounts_iter)?;
            //let event = Container::extract(event_container, accounts_iter)?;
            unimplemented!();
        }
        Instruction::LeaveGame => {
            let mut player = Player::unpack(program_id, accounts_iter)?;
            //FIXME: quick hack caused by the fact, that both player and game are using signer and
            //player_info accounts
            let accounts_iter = &mut accounts.iter();
            let mut game = Game::unpack(program_id, accounts_iter)?;
            game.remove_player(player.data_mut())?;
            Bundle::pack(player)?;
            Bundle::pack(game).map_err(ProgramError::from)
        }
    }
}

// It is old code, that I'm keepeng for easy reference access. I'll delete it as soon as I finish
// instruction implementation

//match tag {
//1 => {
//let game_info = next_account_info(accounts_iter)?;
//let player_state_info = next_account_info(accounts_iter)?;
//let item_ids = Vec::<u32>::deserialize(&mut data)?;
//let mut items: Vec<(u32, &AccountInfo)> = Vec::new();
//for &item_id in &item_ids {
//let item_account_info = next_account_info(accounts_iter)?;
//let item: (u32, &AccountInfo) = (item_id, item_account_info);
//items.push(item);
//}
//// msg!("{:?}", items);
//join_game(game_info, player_state_info, &items)
//}
//2 => {
//let game_info = next_account_info(accounts_iter)?;
//let game_state_info = next_account_info(accounts_iter)?;
//let buf = &mut data;
//let state_step = u32::deserialize(buf)?;
//set_state(game_info, game_state_info, state_step, buf)
//}
//5 => {
//let player_state_info = next_account_info(accounts_iter)?;
//create_player_state(payer_info, player_state_info)
//}
//_ => Err(ProgramError::InvalidAccountData),
//}

//pub fn set_state(
//game_info: &AccountInfo,
//game_state_info: &AccountInfo,
//state_step: u32,
//new_state: &[u8],
//) -> ProgramResult {
//let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
//if game.state_step != state_step {
//return Err(ProgramError::InvalidAccountData);
//}
//game.state_step = state_step + 1;
//game_state_info.data.borrow_mut().copy_from_slice(new_state);
//game.serialize(&mut *game_info.data.borrow_mut())
//.map_err(ProgramError::from)
//}
