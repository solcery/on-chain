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
    msg,
};
use crate::brick::{
    Action,
    Context,
};
use crate::error::SolceryError;
use crate::instruction::SolceryInstruction;
use crate::board::Board;
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
        SolceryInstruction::CreateBoard  => {
            msg!("Instruction: Create Board");
            process_create_board(accounts, program_id)
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
    let mint_account = next_account_info(accounts_iter)?; 

    let expected_card_account_pubkey = Pubkey::create_with_seed(
        mint_account.key,
        "SOLCERYCARD",
        &id()
    )?;
    if expected_card_account_pubkey != *card_account.key {
        return Err(ProgramError::InvalidAccountData);
    }
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
    let card_metadata_account = next_account_info(accounts_iter)?;

    let board = Board::deserialize(&mut &board_account.data.borrow_mut()[..])?;
    let player_info = board.get_player_by_id(*payer_account.key).ok_or(SolceryError::NotAPlayer)?;
    let card_info = board.get_card_by_id(card_id).ok_or(SolceryError::WrongCard)?;
    if card_info.borrow().card_type != *card_metadata_account.key {
        return Err(SolceryError::WrongCard.into())
    }
    if player_info.borrow().attrs[0] == 0 {
        return Err(SolceryError::InGameError.into()) // Player inactive (enemy turn)
    }
    let client_metadata_size = u32::from_le_bytes(card_metadata_account.data.borrow()[..4].try_into().unwrap());
    let mut action = Action::try_from_slice(&card_metadata_account.data.borrow()[client_metadata_size as usize + 4..]).unwrap();    
    let ctx: &mut Context = &mut Context{ 
         object: card_info,
         board: board,
    };
    action.run(ctx);
    ctx.board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_create_board(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let payer_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let board = Board::new();
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
        attrs: [0, 0, 0],
    })));
    if board.players.len() > 1 {
        board.start();
    }
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}
