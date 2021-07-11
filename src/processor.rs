use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    program::{invoke, invoke_signed},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    system_instruction,
    system_program,
    pubkey::Pubkey,
    msg,
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
};
use crate::ruleset::Ruleset;
use crate::player::Player;
use std::convert::TryInto;
use std::io::Write;
use std::rc::Rc;
use std::cell::{
    RefCell,
    RefMut,
};
use crate::card::{
    Card,
    CardType
};

pub enum EntityType {
    Custom,
    Card,
    Ruleset,
    Collection,
}

impl EntityType {
    pub fn from_u8(value: u8) -> EntityType {
        match value {
            1 => EntityType::Card,
            2 => EntityType::Ruleset,
            3 => EntityType::Collection,
            _ => EntityType::Custom,
        }
    }

    pub fn get_name(&self) -> &[u8] {
        match self {
            EntityType::Custom => b"custom",
            EntityType::Card => b"card",
            EntityType::Ruleset => b"ruleset",
            EntityType::Collection => b"collection",
        }
    }
}

declare_id!("5Ds6QvdZAqwVozdu2i6qzjXm8tmBttV6uHNg4YU8rB1P");

pub fn validate_pointer(pointer: &AccountInfo, object: &AccountInfo ) -> bool {
    // msg!("poiner: {:?}, object.key: {:?}", pointer.data.borrow(), object.key.to_bytes());
    return **pointer.data.borrow() == object.key.to_bytes();
}

entrypoint!(process_instruction);
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolceryInstruction::unpack(instruction_data)?;
    match instruction {
        SolceryInstruction::SetEntity { data } => {
            msg!("instruction: SetEntity");
            process_set_entity(accounts, program_id, data)
        }
        SolceryInstruction::DeleteEntity => {
            process_delete_entity(accounts, program_id)
        }
        SolceryInstruction::Cast { card_id } => {
            process_cast(accounts, program_id, card_id)
        }
        SolceryInstruction::CreateBoard => {
            process_create_board(accounts, program_id)
        }
        SolceryInstruction::JoinBoard  => {
            process_join_board(accounts, program_id)
        }
    }
}


pub fn process_set_entity( // TODO:: To create_entity?
    accounts: &[AccountInfo], 
    _program_id: &Pubkey, 
    entity_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let entity_account = next_account_info(accounts_iter)?;
    
    // validation. skipped for now
    // let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap()); // Skipping card visualisation data
    // data = &data[client_metadata_size as usize + 4..];
    entity_account.data.borrow_mut().write_all(&entity_data[..])?;
    Ok(())
}

pub fn process_delete_entity(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let entity_account = next_account_info(accounts_iter)?;
    **payer_account.lamports.borrow_mut() = payer_account.lamports() + entity_account.lamports();
    **entity_account.lamports.borrow_mut() = 0;
    *entity_account.data.borrow_mut() = &mut [];
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
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let ruleset_pointer_account = next_account_info(accounts_iter)?;
    let ruleset_data_account = next_account_info(accounts_iter)?;
    if !validate_pointer(ruleset_pointer_account, ruleset_data_account) {
        return Err(SolceryError::InvalidInstruction.into());
    }
    let ruleset = Ruleset::deserialize(&mut &ruleset_data_account.data.borrow_mut()[..])?;
    let board = {
        let mut cards = Vec::new();
        let mut card_types = Vec::new();
        let mut card_id = 0;
        let mut card_type_id = 0;
        for card_type in ruleset.card_types.iter() {
            let card_pointer_account = next_account_info(accounts_iter)?;
            let card_data_account = next_account_info(accounts_iter)?;
            if !validate_pointer(card_pointer_account, card_data_account) {
                return Err(SolceryError::InvalidInstruction.into());
            }
            card_types.push(Rc::new(RefCell::new(
                CardType::new(card_type_id, card_data_account)
            )));
            card_type_id += 1;
        }
        for place in ruleset.deck.iter() {
            let place_id = place.0;
            let index_amounts = &place.1;
            for card in index_amounts.iter() {
                for i in 0..card.1 {
                    cards.push(Rc::new(RefCell::new(Card {
                        id: card_id,
                        card_type: card.0,
                        place: place_id,
                    })));
                    card_id += 1;
                }
            }
        }
        Board {
            cards: cards,
            card_types: card_types,
            players: Vec::new(),
        }
    };
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
        attrs: [1, 20, 0],
    })));
    if board.players.len() > 0 {
        board.start();
    }
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}
