//! A small colection of utilities used for testing code with account-fs

use account_fs::FS;
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccountParams {
    pub owner: Pubkey,
    pub data: Data,
    pub is_signer: bool,
    pub is_writable: bool,
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
            key: Pubkey::new_unique(),
            lamports: 1,
            data,
            owner: params.owner,
        }));

    AccountInfo::new(
        &internal_info.key,
        params.is_signer,
        params.is_writable,
        &mut internal_info.lamports,
        &mut internal_info.data,
        &internal_info.owner,
        false,
        1,
    )
}

pub fn prepare_fs(program_id: &Pubkey) -> FS<'static, 'static> {
    let params = AccountParams {
        owner: *program_id,
        data: Data::Empty(10_000),
        is_signer: false,
        is_writable: true,
    };

    let mut accounts = Vec::new();
    for _ in 0..3 {
        accounts.push(prepare_account_info(params.clone()));
    }

    let accounts: &'static mut [AccountInfo] = accounts.leak();

    FS::from_uninit_account_iter(&program_id, &mut accounts.iter(), 10).unwrap()
}
