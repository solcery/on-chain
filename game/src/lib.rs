use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

mod game;
use game::{Event, Game, GameState};

mod player;
use player::{Player, PlayerState, CURRENT_PLAYER_VERSION};

mod container;
use container::{Container, Extractable};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
//TODO: Add conversion tests
enum Instruction {
    CreatePlayerAccount,
    UpdatePlayerAccount,
    CreateGame { num_players: u32, max_items: u32 },
    JoinGame,
    AddItems { items_number: u32 },
    SetGameState { new_game_state: GameState },
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
    let payer_info = next_account_info(accounts_iter)?;
    match instruction {
        Instruction::CreatePlayerAccount => {
            let signer = next_account_info(accounts_iter)?;
            let player_info = next_account_info(accounts_iter)?;
            create_player_account(program_id, signer, player_info)
        }
        Instruction::UpdatePlayerAccount => {
            let signer = next_account_info(accounts_iter)?;
            let player_info = next_account_info(accounts_iter)?;
            update_player_account(signer, player_info)
        }
        Instruction::CreateGame {
            num_players,
            max_items,
        } => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game_project = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            create_game(signer, player, game_project, game, num_players, max_items)
        }
        Instruction::JoinGame => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            join_game(signer, player, game)
        }
        Instruction::AddItems { items_number } => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            let mut items = Vec::<&AccountInfo>::with_capacity(items_number as usize);
            for _ in 0..items_number {
                let item_account_info = next_account_info(accounts_iter)?;
                items.push(item_account_info);
            }
            add_items(signer, player, game, items)
        }
        Instruction::SetGameState { new_game_state } => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            set_game_state(signer, player, game)
        }
        Instruction::AddEvent(event_container) => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            let event = Extractable::extract(event_container, accounts_iter)?;
            add_event(signer, player, game, event)
        }
        Instruction::LeaveGame => {
            let signer = next_account_info(accounts_iter)?;
            let player = next_account_info(accounts_iter)?;
            let game = next_account_info(accounts_iter)?;
            leave_game(signer, player, game)
        }
    }
}

fn create_player_account(
    program_id: &Pubkey,
    signer: &AccountInfo,
    player_info: &AccountInfo,
) -> ProgramResult {
    //player_info address check
    let (pda, _bump_seed) =
        Pubkey::find_program_address(&[b"player", signer.key.as_ref()], program_id);

    let mut data: &mut [u8] = &mut player_info.data.borrow_mut();
    let mut buf = &*data;

    //Check previous versions
    let version = <u32>::deserialize(&mut buf);
    match version {
        Ok(1) => {
            //TODO: Better errors
            Player::deserialize(&mut buf)
                //Here error occurs if player account was already initialized
                .map_or(Ok(()), |_| Err(ProgramError::InvalidAccountData))?;
        }
        _ => {}
    }

    if *player_info.key == pda && signer.is_signer {
        let player = Player::from_pubkey(pda);
        (CURRENT_PLAYER_VERSION, player)
            .serialize(&mut data)
            .map_err(ProgramError::from)
    } else {
        Err(ProgramError::InvalidAccountData)
    }
}

fn update_player_account(signer: &AccountInfo, player_info: &AccountInfo) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn create_game(
    signer: &AccountInfo,
    player_info: &AccountInfo,
    game_project: &AccountInfo,
    game: &AccountInfo,
    num_players: u32,
    max_items: u32,
) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn join_game(signer: &AccountInfo, player: &AccountInfo, game: &AccountInfo) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn add_items(
    signer: &AccountInfo,
    player: &AccountInfo,
    game: &AccountInfo,
    items: Vec<&AccountInfo>,
) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn set_game_state(signer: &AccountInfo, player: &AccountInfo, game: &AccountInfo) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn leave_game(signer: &AccountInfo, player: &AccountInfo, game: &AccountInfo) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

fn add_event(
    signer: &AccountInfo,
    player: &AccountInfo,
    game: &AccountInfo,
    event: Event,
) -> ProgramResult {
    //TODO: accounts check
    unimplemented!();
}

#[cfg(test)]
mod tests;

// It is old code, that I'm keepeng for easy reference access. I'll delete it as soon as I finish
// instruction implementation

//match tag {
//0 => {
//let game_info = next_account_info(accounts_iter)?;
//let game_project_info = next_account_info(accounts_iter)?;
//let game_state_info = next_account_info(accounts_iter)?;
//create_game(game_info, game_project_info, game_state_info)
//}
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
//4 => {
//let game_info = next_account_info(accounts_iter)?;
//let player_state_info = next_account_info(accounts_iter)?;
//let victory = bool::deserialize(&mut data)?;
//leave_game(game_info, player_state_info, victory)
//}
//5 => {
//let player_state_info = next_account_info(accounts_iter)?;
//create_player_state(payer_info, player_state_info)
//}
//_ => Err(ProgramError::InvalidAccountData),
//}

//pub fn create_game(
//game_info: &AccountInfo,
//game_project_info: &AccountInfo,
//game_state_info: &AccountInfo,
//) -> ProgramResult {
//let game = Game {
//game_project: *game_project_info.key,
//state_pubkey: *game_state_info.key,
//state_step: 0,
//players: Vec::new(),
//finished: false,
//winners: Vec::new(),
//};
//game.serialize(&mut *game_info.data.borrow_mut())
//.map_err(ProgramError::from)
//}

//pub fn join_game(
//game_info: &AccountInfo,
//player_state_info: &AccountInfo,
//items: &[(u32, &AccountInfo)],
//) -> ProgramResult {
//let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
//let mut player_state = PlayerState::deserialize(&mut player_state_info.data.borrow().as_ref())?;
//let player_items = items.iter().map(|item| (item.0, *item.1.key)).collect();
//msg!("{:?}", player_items);
//game.players.push(Player {
//pubkey: player_state.pubkey,
//online: true,
//items: player_items,
//});
//player_state.game = Some(*game_info.key);
//player_state
//.serialize(&mut *player_state_info.data.borrow_mut())
//.map_err(ProgramError::from)?;
//game.serialize(&mut *game_info.data.borrow_mut())
//.map_err(ProgramError::from)
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

//pub fn leave_game(
//game_info: &AccountInfo,
//player_state_info: &AccountInfo,
//_victory: bool,
//) -> ProgramResult {
//let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
//let mut player_state = PlayerState::deserialize(&mut player_state_info.data.borrow().as_ref())?;

//game.finished = true;
//for player in &mut game.players {
//if player.pubkey == player_state.pubkey {
//player_state.game = None;
//player.online = false;
//player_state
//.serialize(&mut *player_state_info.data.borrow_mut())
//.map_err(ProgramError::from)?;
//return game
//.serialize(&mut *game_info.data.borrow_mut())
//.map_err(ProgramError::from);
//}
//}
//Err(ProgramError::InvalidAccountData)
//}

//pub fn create_player_state(
//payer_info: &AccountInfo,
//player_state_info: &AccountInfo,
//) -> ProgramResult {
//let player_state = PlayerState {
//pubkey: *payer_info.key,
//game: None,
//};
//player_state
//.serialize(&mut *player_state_info.data.borrow_mut())
//.map_err(ProgramError::from)
//}
