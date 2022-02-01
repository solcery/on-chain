use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Bundled<'a, T> {
    data: T,
    accounts: Vec<AccountInfo<'a>>,
}

impl<'a, T> Borrow<T> for Bundled<'a, T> {
    #[inline]
    fn borrow(&self) -> &T {
        &self.data
    }
}

impl<'a, T> BorrowMut<T> for Bundled<'a, T> {
    #[inline]
    fn borrow_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<'a, T: PartialEq> PartialEq for Bundled<'a, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

pub trait Bundle<'a>
where
    Self: Sized,
{
    type Error;
    fn new(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
    ) -> Result<Bundled<'a, Self>, ProgramError>;
    fn unpack(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
    ) -> Result<Bundled<'a, Self>, ProgramError>;
    fn pack(bundle: Bundled<'a, Self>) -> Result<(), Self::Error>;
}
