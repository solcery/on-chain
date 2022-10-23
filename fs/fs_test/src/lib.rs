//! A small colection of utilities used for testing code with account-fs

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{account_info::AccountInfo, pubkey::Pubkey};

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct AccountParams {
    pub address: Option<Pubkey>,
    pub owner: Pubkey,
    pub data: AccountData,
}

/// This struct is used to store data, which is borrowed in ordinal [AccountInfo]
#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct InternalAccountInfo {
    key: Pubkey,
    lamports: u64,
    data: Vec<u8>,
    owner: Pubkey,
}

impl InternalAccountInfo {
    pub fn account_info<'a>(&'a mut self) -> AccountInfo<'a> {
        AccountInfo::new(
            &self.key,
            false,
            true,
            &mut self.lamports,
            &mut self.data,
            &self.owner,
            false,
            1,
        )
    }

    pub fn from_account_params(params: AccountParams) -> Self {
        let data = match params.data {
            AccountData::Filled(vec) => vec,
            AccountData::Empty(cap) => vec![0; cap],
        };

        Self {
            key: params.address.unwrap_or(Pubkey::new_unique()),
            lamports: 1,
            data,
            owner: params.owner,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum AccountData {
    Filled(Vec<u8>),
    Empty(usize),
}

#[derive(Clone, Debug, Eq, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct FSAccounts(pub Vec<InternalAccountInfo>);

impl FSAccounts {
    pub fn replicate_params(params: AccountParams, count: usize) -> Self {
        let accounts = std::iter::repeat(params)
            .take(count)
            .map(InternalAccountInfo::from_account_params)
            .collect();
        Self(accounts)
    }

    pub fn account_info_iter<'a>(&'a mut self) -> Vec<AccountInfo<'a>> {
        self.0
            .iter_mut()
            .map(|internal_info| internal_info.account_info())
            .collect()
    }

    pub fn owner_pubkey(&self) -> Option<Pubkey> {
        self.0.get(0).map(|x| x.owner)
    }
}
