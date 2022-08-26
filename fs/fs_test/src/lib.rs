//! A small colection of utilities used for testing code with account-fs

use account_fs::{SegmentId, FS};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountParams {
    pub address: Option<Pubkey>,
    pub owner: Pubkey,
    pub data: Data,
}

/// This struct is used to store data, which is borrowed in ordinal AccountInfo<'_>
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InternalAccountInfo {
    key: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Data {
    Filled(Vec<u8>),
    Empty(usize),
}

/// Due to the way, how this function works, it causes memory leaks
pub fn prepare_account_info(params: AccountParams) -> AccountInfo<'static> {
    let data = match params.data {
        Data::Filled(vec) => vec,
        Data::Empty(cap) => vec![0; cap],
    };

    let internal_info: &'static mut InternalAccountInfo =
        Box::leak(Box::new(InternalAccountInfo {
            key: params.address.unwrap_or(Pubkey::new_unique()),
            lamports: 1,
            data,
            owner: params.owner,
        }));

    AccountInfo::new(
        &internal_info.key,
        false,
        true,
        &mut internal_info.lamports,
        &mut internal_info.data,
        &internal_info.owner,
        false,
        1,
    )
}

pub fn prepare_fs(program_id: &Pubkey) -> FS<'static, 'static> {
    let params = AccountParams {
        address: None,
        owner: *program_id,
        data: Data::Empty(10_000),
    };

    let mut accounts = Vec::new();
    for _ in 0..3 {
        accounts.push(prepare_account_info(params.clone()));
    }

    let accounts: &'static mut [AccountInfo] = accounts.leak();

    FS::from_uninit_account_iter(&program_id, &mut accounts.iter(), 10).unwrap()
}

pub fn prepare_raw_fs<AccountIter>(
    program_id: &Pubkey,
    accounts: AccountIter,
) -> FS<'static, 'static>
where
    AccountIter: IntoIterator<Item = AccountParams>,
{
    let mut generated_accounts = Vec::new();
    for account in accounts {
        generated_accounts.push(prepare_account_info(account));
    }

    let accounts: &'static mut [AccountInfo] = generated_accounts.leak();

    FS::from_uninit_account_iter(&program_id, &mut accounts.iter(), 10).unwrap()
}
