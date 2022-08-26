//! A small colection of utilities used for testing code with account-fs

use account_fs::{SegmentId, FS};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountParams {
    pub address: Option<Pubkey>,
    pub owner: Pubkey,
    pub data: AccountData,
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
pub enum AccountData {
    Filled(Vec<u8>),
    Empty(usize),
}

/// Due to the way, how this function works, it causes memory leaks
pub fn prepare_account_info(params: AccountParams) -> AccountInfo<'static> {
    let data = match params.data {
        AccountData::Filled(vec) => vec,
        AccountData::Empty(cap) => vec![0; cap],
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
        data: AccountData::Empty(10_000),
    };

    prepare_raw_fs(program_id, std::iter::repeat(params).take(4))
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

    FS::from_uninit_account_iter(program_id, &mut accounts.iter(), 10).unwrap()
}

const ACCOUNT_SIZE: usize = 100_000;
pub fn prepare_fs_from_segments<SegmentIter>(
    program_id: &Pubkey,
    accounts: SegmentIter,
) -> FS<'static, 'static>
where
    SegmentIter: IntoIterator<Item = (SegmentId, Vec<u8>)>,
{
    let mut generated_accounts = BTreeMap::new();
    for (segment_id, data) in accounts {
        if let Some(account) = generated_accounts.get(&segment_id.pubkey) {
            todo!();
        } else {
            let account_params = AccountParams {
                address: Some(segment_id.pubkey),
                owner: *program_id,
                data: AccountData::Empty(ACCOUNT_SIZE),
            };
            let mut account = prepare_account_info(account_params);
            generated_accounts.insert(segment_id.pubkey, account);
            todo!();
        }
    }
    todo!();
}
