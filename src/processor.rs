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
    UnitAction,
    Context,
};
use crate::instruction::SolceryInstruction;
use crate::fight::Fight;
use crate::board::Board;
use crate::unit::UnitType;
use std::convert::TryInto;
use std::io::Write;
use crate::board::PlaceId;
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
        SolceryInstruction::Cast { caster_id, position } => {
            msg!("Instruction: Cast");
            process_cast(accounts, program_id, caster_id, position)
        }
        SolceryInstruction::CreateFight  => {
            process_create_fight(accounts, program_id)
        }
        SolceryInstruction::SpawnUnit { position }  => {
            msg!("Instruction: SpawnUnit");
            process_spawn_unit(accounts, program_id, position)
        }
        SolceryInstruction::CreateUnit { data }  => {
            msg!("Instruction: CreateUnit");
            process_create_unit(accounts, program_id, data)
        }
    }
}


pub fn process_create_card(
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
    msg!("CARD DATA: {:?}", card_data);
    let client_metadata_size = u32::from_le_bytes(card_data[..4].try_into().unwrap()); // Skipping card visualisation data
    data = &data[client_metadata_size as usize + 4..];
    let action = UnitAction::try_from_slice(&data[..])?; // 
    msg!("Action: {:?}", action);
    {
        let card_account_data = &mut &mut card_account.data.borrow_mut()[..];
        card_account_data.write_all(&card_data[..])?;
    }
    msg!("CreateCard: {:?}", &card_account.data.borrow()[..]);
    Ok(())
}

pub fn process_create_unit(
    accounts: &[AccountInfo], 
    _program_id: &Pubkey, 
    unit_data: Vec<u8>,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let _payer_account = next_account_info(accounts_iter)?; // ignored, we don't check card ownership for now
    let unit_account = next_account_info(accounts_iter)?;
    let mint_account = next_account_info(accounts_iter)?; 

    let expected_unit_account_pubkey = Pubkey::create_with_seed(
        mint_account.key,
        "SOLCERYUNIT",
        &id()
    )?;
    if expected_unit_account_pubkey != *unit_account.key {
        return Err(ProgramError::InvalidAccountData);
    }
    let mut data = &unit_data[..];
    let client_metadata_size = u32::from_le_bytes(unit_data[..4].try_into().unwrap()); // Skipping unit visualisation data
    data = &data[client_metadata_size as usize + 4..];
    let unit_type = UnitType::try_from_slice(&data[..])?; // 
    let unit_account_data = &mut &mut unit_account.data.borrow_mut()[..];
    unit_account_data.write_all(&unit_data[..])?;
    Ok(())
}

pub fn process_cast(
    accounts: &[AccountInfo],
    _program_id: &Pubkey,
    caster_id: u32,
    position: PlaceId,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let _payer_account = next_account_info(accounts_iter)?;
    let _fight_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let card_metadata_account = next_account_info(accounts_iter)?;
    let client_metadata_size = u32::from_le_bytes(card_metadata_account.data.borrow()[..4].try_into().unwrap());
    msg!("PROCESS CAST: {:?}", &card_metadata_account.data.borrow()[client_metadata_size as usize + 4..]);
    let mut action = UnitAction::try_from_slice(&card_metadata_account.data.borrow()[client_metadata_size as usize + 4..]).unwrap();
    msg!("{:?}", action);
    //let fight = Fight::try_from_slice(&fight_account.data.borrow()[..])?; 
    let ctx: &mut Context = &mut Context{ 
         objects: Vec::new(),
         place: position,
         board: Board::deserialize(&mut &board_account.data.borrow_mut()[..])?,
    };
    ctx.objects.push(ctx.board.get_unit_by_id(caster_id).unwrap());
    action.run(ctx);
    ctx.board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    msg!("Board serialized! {:?}", board_account.data.borrow());
    Ok(())
}

pub fn process_create_fight(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer_account = next_account_info(accounts_iter)?;
    let fight_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?; 

    let fight = Fight::new(*payer_account.key);
    fight.serialize(&mut &mut fight_account.data.borrow_mut()[..])?;
    let board = Board::new(8, 8);
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;
    Ok(())
}

pub fn process_spawn_unit(
    accounts: &[AccountInfo],
    program_id: &Pubkey, // Public key of the account the program was loaded into
    position: PlaceId,
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let payer_account = next_account_info(accounts_iter)?;
    let _fight_account = next_account_info(accounts_iter)?;
    let board_account = next_account_info(accounts_iter)?;
    let unit_metadata_account = next_account_info(accounts_iter)?;

    let mut board = Board::deserialize(&mut &board_account.data.borrow()[..])?;
    board.create_unit(*payer_account.key, *unit_metadata_account.key, position);
    board.serialize(&mut &mut board_account.data.borrow_mut()[..])?;

    Ok(())
}
