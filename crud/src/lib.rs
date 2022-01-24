use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    program_pack::IsInitialized,
    pubkey::Pubkey,
};

/// Struct wrapping data and providing metadata
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct RecordData {
    /// Struct version, allows for upgrades to the program
    pub version: u8,

    /// Project which owns the account
    pub project: Pubkey,

    /// The data contained by the account, could be anything serializable
    pub data: Data,
}

/// Struct just for data
#[derive(Clone, Debug, Default, BorshSerialize, BorshDeserialize, BorshSchema, PartialEq)]
pub struct Data {
    /// The data contained by the account, could be anything or serializable
    pub bytes: Vec<u8>,
}

pub fn process_instruction(accounts: &[AccountInfo], instruction_data: &[u8]) -> ProgramResult {
    let (tag, data) = instruction_data.split_first().unwrap();
    let accounts_iter = &mut accounts.iter();
    msg!("Crud/Process");
    match tag {
        0 => {
            let account_info = next_account_info(accounts_iter)?;
            write_raw(account_info, 0, data.to_vec())
        }
        _ => Err(ProgramError::InvalidAccountData),
    }
}

impl RecordData {
    /// Version to fill in on new created accounts
    pub const CURRENT_VERSION: u8 = 1;

    /// Start of writable account data, after version and authority
    pub const WRITABLE_START_INDEX: usize = 33;
}

impl IsInitialized for RecordData {
    /// Is initialized
    fn is_initialized(&self) -> bool {
        self.version == Self::CURRENT_VERSION
    }
}

pub fn initialize(project_info: &AccountInfo, account_info: &AccountInfo) -> ProgramResult {
    let mut account_data = RecordData::deserialize(&mut &account_info.data.borrow()[..])?;
    if account_data.is_initialized() {
        return Err(ProgramError::AccountAlreadyInitialized);
    }
    account_data.version = RecordData::CURRENT_VERSION;
    account_data.project = *project_info.key;
    account_data
        .serialize(&mut &mut account_info.data.borrow_mut()[..])
        .map_err(|e| e.into())
}

pub fn write_raw(account_info: &AccountInfo, offset: u64, mut data: Vec<u8>) -> ProgramResult {
    msg!("Crud/Write raw");
    let start = 0;
    let end = start + data.len();
    if end > account_info.data.borrow().len() {
        Err(ProgramError::AccountDataTooSmall)
    } else {
        let rest = account_info.data.borrow().len() - end;
        let mut zeroes: Vec<u8> = vec![0; rest];
        data.append(&mut zeroes);
        account_info.data.borrow_mut()[..].copy_from_slice(&data);
        Ok(())
    }
}

pub fn write(account_info: &AccountInfo, offset: u64, mut data: Vec<u8>) -> ProgramResult {
    msg!("Crud/Write");
    // let account_data = RecordData::deserialize(&mut &account_info.data.borrow()[..])?; //TODO
    // if !account_data.is_initialized() {
    //     msg!("Not init");
    //     return Err(ProgramError::UninitializedAccount);
    // }
    let start = RecordData::WRITABLE_START_INDEX + offset as usize;
    let end = start + data.len();
    if end > account_info.data.borrow().len() {
        Err(ProgramError::AccountDataTooSmall)
    } else {
        let rest = account_info.data.borrow().len() - end;
        let mut zeroes: Vec<u8> = vec![0; rest];
        data.append(&mut zeroes);
        account_info.data.borrow_mut()[start..].copy_from_slice(&data);
        Ok(())
    }
}
