use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    instruction::{AccountMeta, Instruction},
    program::{invoke, invoke_signed},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    system_instruction,
    system_program,
    pubkey::Pubkey,
    msg,
    sysvar::{ clock::Clock, Sysvar },
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
    Log,
};
use crate::fight_log::{
    FightLog,
    LogEntry
};
use crate::ruleset::Ruleset;
use crate::player::Player;
use std::convert::TryInto;
use std::io::Write;
use std::rc::Rc;
use crate::rand::Rand;
use std::cell::{
    RefCell,
    RefMut,
};
use crate::card::{
    Card,
    CardType
};
use std::str::FromStr;
use std::collections::LinkedList;

pub enum EntityType {
    Custom,
    Card,
    Ruleset,
    Collection,
}

#[derive(BorshSerialize, BorshDeserialize)]
struct Lobby {
    pub boards: Vec<[u8; 32]>,
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

//НЕ работает покупка, подпись на апдейты, ворона сильно дамажит

pub fn validate_pointer(pointer: &AccountInfo, object: &AccountInfo ) -> bool {
    // msg!("poiner: {:?}, object.key: {:?}", pointer.data.borrow(), object.key.to_bytes());
    return **pointer.data.borrow() == object.key.to_bytes();
}

entrypoint!(process_instruction);
declare_id!("4YyCGiiZ3EorWmcQs3yrCRfTGt8udhDvV9ffJoWJaXUX");
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let instruction = SolceryInstruction::unpack(instruction_data)?;
    match instruction {
        SolceryInstruction::SetEntity { position, data } => {
            process_set_entity(accounts, program_id, position, data)
        }
        SolceryInstruction::DeleteEntity => {
            process_delete_entity(accounts, program_id)
        }
        SolceryInstruction::AddLog { log } => {
            process_add_log(accounts, program_id, log)
        }
        SolceryInstruction::CreateBoard { random_seed } => {
            process_create_board(accounts, program_id, random_seed)
        }
        SolceryInstruction::AddCardsToBoard { cards_amount } => {
            process_add_cards_to_board(accounts, program_id, cards_amount)
        }
        SolceryInstruction::JoinBoard  => {
            process_join_board(accounts, program_id)
        }
    }
}


pub fn process_set_entity( // TODO:: To create_entity?
    accounts: &[AccountInfo], 
    _program_id: &Pubkey, 
    position: u32,
    entity_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let entity_account = next_account_info(accounts_iter)?;
    
    // validation. skipped for now
    // let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap()); // Skipping card visualisation data
    // data = &data[client_metadata_size as usize + 4..];
    let y = &mut &mut entity_account.data.borrow_mut()[position as usize..position as usize + entity_data.len()];
    for i in 0..entity_data.len() {
        y[i] = entity_data[i];
    }
    // let mut x = entity_account.data.borrow_mut()[position as usize..].to_vec();
    // x.write_all(&entity_data[..])?;
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


pub fn process_add_log(
    accounts: &[AccountInfo],
    program_id: &Pubkey,
    log: FightLog,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;


    // let caster_id = board.get_player_index_by_id(*payer_account.key);
    // {
    //     let board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    //     let log_iter = log.log.iter();
    //     for log_entry in log_iter {
    //         let player_info = board.get_player(log_entry.player_id).unwrap();
    //         if (player_info.borrow().id != *payer_account.key && player_info.borrow().id != *board_account.key) {
    //             return Err(SolceryError::InGameError.into()) // Trying to add log for other player
    //         }
    //     }
    // }
    let fight_log_account = next_account_info(accounts_iter)?;
    // let mut fight_log = FightLog::deserialize(&mut &fight_log_account.data.borrow_mut()[..])?;
    // let log_iter = log.log.iter();
    // for log_entry in log_iter {
    //         fight_log.log.push(LogEntry {
    //         player_id: log_entry.player_id,
    //         action_type: log_entry.action_type,
    //         action_data: log_entry.action_data,
    //     });
    //     fight_log_account.data.borrow_mut()[..
    // }
    // fight_log_account.data.borrow_mut()[..
    // fight_log.serialize(&mut &mut fight_log_account.data.borrow_mut()[..])?;



    let log_size = u32::try_from_slice(&fight_log_account.data.borrow()[..4])?;
    msg!("{:?}", log_size);
    let mut offset = log_size * 12 + 4;
    let log_iter = log.log.iter();
    for log_entry in log_iter {
        let new_log = LogEntry {
            player_id: log_entry.player_id,
            action_type: log_entry.action_type,
            action_data: log_entry.action_data,
        };
        new_log.serialize(&mut &mut fight_log_account.data.borrow_mut()[offset as usize..offset as usize + 12]);
        offset += 12;
    }
    (log_size + log.log.len() as u32).serialize(&mut &mut fight_log_account.data.borrow_mut()[..4]);
    // if (card_id > 7) {
    //     let ix = Instruction {
    //         program_id: id(),
    //         accounts: vec![ 
    //             AccountMeta::new(*payer_account.key, false ),
    //             AccountMeta::new(id(), false ),
    //         ],
    //         data: vec![5, (card_id-1).try_into().unwrap(), 0, 0, 0],
    //     };
    //     invoke(
    //         &ix,
    //         &[
    //             program_account.clone(),
    //             payer_account.clone(),
    //         ],
    //     )?;
    // }
    
    // let card_info = board.get_card_by_id(card_id).ok_or(SolceryError::WrongCard)?;
    // if player_info.borrow().attrs[0] == 0 {
    //     return Err(SolceryError::InGameError.into()) // Player inactive (enemy turn)
    // }
    // let caster_id = board.get_player_index_by_id(*payer_account.key);
    // board.cast_card(card_id, caster_id);

    // if (board.players[1].borrow().attrs[12] > 0) { //bot behaviour
    //     if (board.players[1].borrow().attrs[0] > 0) { //bot is active
    //         board.cast_card(1, 2);
    //     }
    // }
    // msg!("epoch {:?}", Clock::get().unwrap());
    
    // board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;

    Ok(())
}

pub fn process_create_board( 
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
    random_seed: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let stat_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let fight_log_account = next_account_info(accounts_iter)?;
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
            last_update: Clock::get().unwrap().unix_timestamp.try_into().unwrap(),
            step: 0,
            cards: cards,
            card_types: card_types,
            players: Vec::new(),
            log: Rc::new(RefCell::new(Log {
                message_len: 0,
                nonce: 0,
                message: [0; 128],
            })),
            rand: Rc::new(RefCell::new(Rand::new(random_seed))),
        }
    };
    let mut stat = Lobby::deserialize(&mut &stat_account.data.borrow_mut()[..])?;
    stat.boards.push(board_account.key.to_bytes());
    stat.serialize(&mut &mut stat_account.data.borrow_mut()[..])?;
    
    let fight_log = FightLog {
        log: Vec::new(),
    };
    fight_log.serialize(&mut &mut fight_log_account.data.borrow_mut()[..])?;

    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_add_cards_to_board( 
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
    cards_amount: u32,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let ruleset_pointer_account = next_account_info(accounts_iter)?;
    let ruleset_data_account = next_account_info(accounts_iter)?;
    if !validate_pointer(ruleset_pointer_account, ruleset_data_account) {
        return Err(SolceryError::InvalidInstruction.into());
    }
    // let ruleset = Ruleset::deserialize(&mut &ruleset_data_account.data.borrow_mut()[..])?;
    let mut board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    for i in 1..cards_amount + 1 { // TODO: check validity
        let card_pointer_account = next_account_info(accounts_iter)?;
        let card_data_account = next_account_info(accounts_iter)?;
        // if !validate_pointer(card_pointer_account, card_data_account) {
        //     return Err(SolceryError::InvalidInstruction.into());
        // }
        board.card_types.push(Rc::new(RefCell::new(
            CardType::new(board.card_types.len().try_into().unwrap(), card_data_account)
        )));
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
    let lobby_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?; 
    let fight_log_account = next_account_info(accounts_iter)?;
    // msg!("deserialize {:?}", &board_account.data.borrow()[..100]);
    let mut board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    let mut fight_log = FightLog::deserialize(&mut &fight_log_account.data.borrow_mut()[..])?;
    if board.players.len() > 1 {
        // msg!("Too many players");
        return Err(SolceryError::GameStarted.into());
    }
    board.players.push(Rc::new(RefCell::new(Player{
        id: *payer_account.key,
        attrs: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    })));
    fight_log.log.push( LogEntry {
        player_id: (board.players.len()).try_into().unwrap(),
        action_type: 1,
        action_data: 2,
    });
    {
        board.players.push(Rc::new(RefCell::new(Player{ // bot
            id: *board_account.key,
            attrs: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        })));
        fight_log.log.push( LogEntry {
            player_id: (board.players.len()).try_into().unwrap(),
            action_type: 1,
            action_data: 2,
        });
    }
    if board.players.len() > 1 {
        for card in board.cards.iter() {
            if (card.borrow().place == 0) {
                fight_log.log.push( LogEntry {
                    player_id: 1,
                    action_type: 0,
                    action_data: card.borrow().id,
                });
            }
        }
    }
    fight_log.serialize(&mut &mut fight_log_account.data.borrow_mut()[..])?;

    // if (board.players.len() == 1) {
    //     let mut lobby = Lobby::deserialize(&mut &lobby_account.data.borrow_mut()[..])?;
    //     lobby.boards.push(board_account.key.to_bytes());
    //     lobby.serialize(&mut &mut lobby_account.data.borrow_mut()[..])?;
    // }

    // if (board.players.len() == 2) {
    //     let mut lobby = Lobby::deserialize(&mut &lobby_account.data.borrow_mut()[..])?;
    //     let index = lobby.boards.iter().position(|slice_key| *slice_key == board_account.key.to_bytes());
    //     match index {
    //         Some(index) => {
    //             lobby.boards.remove(index);
    //             lobby.serialize(&mut &mut lobby_account.data.borrow_mut()[..])?;
    //         },
    //         _ => return Err(SolceryError::GameStarted.into()),
    //     }
    // }

    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}
