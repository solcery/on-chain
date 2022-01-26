use borsh::{BorshDeserialize, BorshSerialize};
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

pub trait Extractable
where
    Self: Clone + Eq + PartialEq + Ord + PartialOrd + Debug + BorshSerialize + BorshDeserialize,
{
    fn extract(
        containered_data: Container<Self>,
        accounts_iter: &mut std::slice::Iter<'_, AccountInfo<'_>>,
    ) -> Result<Self, ProgramError>;
}
