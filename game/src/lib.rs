use {
    borsh::{BorshDeserialize, BorshSerialize},
    solana_program::{
        account_info::{next_account_info, AccountInfo},
        entrypoint,
        entrypoint::ProgramResult,
        msg,
        program_error::ProgramError,
        pubkey::Pubkey,
    },
};

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GameObject {
    pub id: u32,
    pub tpl_id: u32,
    pub attrs: Vec<u32>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct GameState {
    objects: Vec<GameObject>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Player {
    pubkey: Pubkey,
    online: bool,
    items: Vec<(u32, Pubkey)>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Game {
    game_project: Pubkey,
    state_pubkey: Pubkey,
    pub state_step: u32,
    players: Vec<Player>,
    finished: bool,
    winners: Vec<Pubkey>,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct PlayerState {
    //TODO: move to SolceryPlayer protocol
    pub pubkey: Pubkey,
    pub game: Option<Pubkey>,
}

entrypoint!(process_instruction);
pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (tag, mut data) = instruction_data.split_first().unwrap();
    let accounts_iter = &mut accounts.iter();
    let payer_info = next_account_info(accounts_iter)?;
    match tag {
        0 => {
            let game_info = next_account_info(accounts_iter)?;
            let game_project_info = next_account_info(accounts_iter)?;
            let game_state_info = next_account_info(accounts_iter)?;
            create_game(game_info, game_project_info, game_state_info)
        }
        1 => {
            let game_info = next_account_info(accounts_iter)?;
            let player_state_info = next_account_info(accounts_iter)?;
            let item_ids = Vec::<u32>::deserialize(&mut data)?;
            let mut items: Vec<(u32, &AccountInfo)> = Vec::new();
            for &item_id in &item_ids {
                let item_account_info = next_account_info(accounts_iter)?;
                let item: (u32, &AccountInfo) = (item_id, item_account_info);
                items.push(item);
            }
            // msg!("{:?}", items);
            join_game(game_info, player_state_info, &items)
        }
        2 => {
            let game_info = next_account_info(accounts_iter)?;
            let game_state_info = next_account_info(accounts_iter)?;
            let buf = &mut data;
            let state_step = u32::deserialize(buf)?;
            set_state(game_info, game_state_info, state_step, buf)
        }
        4 => {
            let game_info = next_account_info(accounts_iter)?;
            let player_state_info = next_account_info(accounts_iter)?;
            let victory = bool::deserialize(&mut data)?;
            leave_game(game_info, player_state_info, victory)
        }
        5 => {
            let player_state_info = next_account_info(accounts_iter)?;
            create_player_state(payer_info, player_state_info)
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

pub fn create_game(
    game_info: &AccountInfo,
    game_project_info: &AccountInfo,
    game_state_info: &AccountInfo,
) -> ProgramResult {
    let game = Game {
        game_project: *game_project_info.key,
        state_pubkey: *game_state_info.key,
        state_step: 0,
        players: Vec::new(),
        finished: false,
        winners: Vec::new(),
    };
    game.serialize(&mut *game_info.data.borrow_mut())
        .map_err(ProgramError::from)
}

pub fn join_game(
    game_info: &AccountInfo,
    player_state_info: &AccountInfo,
    items: &[(u32, &AccountInfo)],
) -> ProgramResult {
    let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
    let mut player_state = PlayerState::deserialize(&mut player_state_info.data.borrow().as_ref())?;
    let player_items = items.iter().map(|item| (item.0, *item.1.key)).collect();
    msg!("{:?}", player_items);
    game.players.push(Player {
        pubkey: player_state.pubkey,
        online: true,
        items: player_items,
    });
    player_state.game = Some(*game_info.key);
    player_state
        .serialize(&mut *player_state_info.data.borrow_mut())
        .map_err(ProgramError::from)?;
    game.serialize(&mut *game_info.data.borrow_mut())
        .map_err(ProgramError::from)
}

pub fn set_state(
    game_info: &AccountInfo,
    game_state_info: &AccountInfo,
    state_step: u32,
    new_state: &[u8],
) -> ProgramResult {
    let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
    if game.state_step != state_step {
        return Err(ProgramError::InvalidAccountData);
    }
    game.state_step = state_step + 1;
    game_state_info.data.borrow_mut().copy_from_slice(new_state);
    game.serialize(&mut *game_info.data.borrow_mut())
        .map_err(ProgramError::from)
}

pub fn leave_game(
    game_info: &AccountInfo,
    player_state_info: &AccountInfo,
    _victory: bool,
) -> ProgramResult {
    let mut game = Game::deserialize(&mut game_info.data.borrow().as_ref())?;
    let mut player_state = PlayerState::deserialize(&mut player_state_info.data.borrow().as_ref())?;

    game.finished = true;
    for player in &mut game.players {
        if player.pubkey == player_state.pubkey {
            player_state.game = None;
            player.online = false;
            player_state
                .serialize(&mut *player_state_info.data.borrow_mut())
                .map_err(ProgramError::from)?;
            return game
                .serialize(&mut *game_info.data.borrow_mut())
                .map_err(ProgramError::from);
        }
    }
    Err(ProgramError::InvalidAccountData)
}

pub fn create_player_state(
    payer_info: &AccountInfo,
    player_state_info: &AccountInfo,
) -> ProgramResult {
    let player_state = PlayerState {
        pubkey: *payer_info.key,
        game: None,
    };
    player_state
        .serialize(&mut *player_state_info.data.borrow_mut())
        .map_err(ProgramError::from)
}
