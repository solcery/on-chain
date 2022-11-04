use solana_program::{
    account_info::{next_account_info, AccountInfo},
    pubkey::Pubkey,
};

use crate::error::Error;
use crate::instruction::Instruction;
use crate::state::{
    bundled::Bundle, container::Container, game::Game, game_state::State, player::Player,
};

pub fn process_instruction(
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
