use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    program::{invoke, invoke_signed},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    system_instruction,
    pubkey::Pubkey,
    declare_id,
};
use crate::brick::{
    Action,
    Context,
};
use std::collections::BTreeMap;
use crate::error::SolceryError;
use crate::instruction::SolceryInstruction;
use crate::board::{
    Board,
    Ruleset,
    Place
};
use crate::player::Player;
use std::convert::TryInto;
use std::io::Write;
use std::rc::Rc;
use std::cell::{
    RefCell,
    RefMut,
};


declare_id!("A1U9yQfGgNMn2tkE5HB576QYoBA3uAdNFdjJA439S4m6");

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolceryInstruction::unpack(instruction_data)?;
    match instruction {
        SolceryInstruction::CreateCard { data } => {
            process_create_card(accounts, program_id, data)
        }
        SolceryInstruction::Cast { card_id } => {
            process_cast(accounts, program_id, card_id)
        }
        SolceryInstruction::CreateBoard { deck, init } => {
            process_create_board(accounts, program_id, deck, init)
        }
        SolceryInstruction::JoinBoard  => {
            process_join_board(accounts, program_id)
        }
    }
}


pub fn process_create_card( // TODO:: To create_entity
    accounts: &[AccountInfo], 
    _program_id: &Pubkey, 
    card_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let card_account = next_account_info(accounts_iter)?;
    let mut data = &card_data[..];
    let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap()); // Skipping card visualisation data
    data = &data[client_metadata_size as usize + 4..];
    let action = Action::try_from_slice(&data[..])?; // 
    {
        let card_account_data = &mut &mut card_account.data.borrow_mut()[..];
        card_account_data.write_all(&card_data[..])?;
    }
    Ok(())
}

pub fn process_cast(
    accounts: &[AccountInfo],
    _program_id: &Pubkey,
    card_id: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;

    let board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    let player_info = board.get_player_by_id(*payer_account.key).ok_or(SolceryError::NotAPlayer)?;
    let card_info = board.get_card_by_id(card_id).ok_or(SolceryError::WrongCard)?;
    if player_info.borrow().attrs[0] == 0 {
        return Err(SolceryError::InGameError.into()) // Player inactive (enemy turn)
    }
    let caster_id = board.get_player_index_by_id(*payer_account.key);
    board.cast_card(card_id, caster_id);
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_create_board(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
    deck: Vec<(u32, Place)>,
    init: Vec<u32>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let mut board_deck = Vec::new();
    for deck_entry in deck.iter() {
        let card_account = next_account_info(accounts_iter)?;
        board_deck.push((card_account.clone(), deck_entry.0, deck_entry.1));
    }
    let board = Board::new( Ruleset{ deck: board_deck } );
    for card_id in init.iter() {
        board.cast_card(*card_id, 0);
    }
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_join_board(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?; 
    let mut board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    if board.players.len() > 1 {
        return Err(SolceryError::GameStarted.into());
    }
    board.players.push(Rc::new(RefCell::new(Player{
        id: *payer_account.key,
        attrs: [0, 20, 0],
    })));
    if board.players.len() > 1 {
        board.start();
    }
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}
