//! # Container type
//!
//! This type is used for data, which can be either stored directly in the instruction_data or in
//! the separate account.
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::account_info::next_account_info;
use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub enum Container<T>
where
    T: Clone + Eq + PartialEq + Ord + PartialOrd + Debug + BorshSerialize + BorshDeserialize,
{
    InPlace(T),
    InAccount(Pubkey),
}

impl<T> Container<T>
where
    T: Clone + Eq + PartialEq + Ord + PartialOrd + Debug + BorshSerialize + BorshDeserialize,
{
    pub fn extract<'short, 'long, AccountIter>(
        containered_data: Self,
        accounts_iter: &mut AccountIter,
    ) -> Result<T, ProgramError>
    where
        AccountIter: Iterator<Item = &'short AccountInfo<'long>>,
        'long: 'short,
    {
        match containered_data {
            Container::InPlace(data) => Ok(data),
            Container::InAccount(pubkey) => {
                let data_account = next_account_info(accounts_iter)?;
                if *data_account.key == pubkey {
                    let data = T::deserialize(&mut data_account.data.borrow().as_ref())?;
                    Ok(data)
                } else {
                    //TODO: We need more descriptive error here
                    Err(ProgramError::InvalidAccountData)
                }
            }
        }
    }
}
