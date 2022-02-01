use solana_program::account_info::AccountInfo;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Bundled<'a, T> {
    data: T,
    accounts: Vec<&'a AccountInfo<'a>>,
}

impl<'a, T> Bundled<'a, T> {
    pub unsafe fn new(data: T, accounts: Vec<&'a AccountInfo<'a>>) -> Self {
        Self { data, accounts }
    }
    pub unsafe fn release(self) -> (T, Vec<&'a AccountInfo<'a>>) {
        (self.data, self.accounts)
    }
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

pub trait Bundle<'a, InitializationArg>
where
    Self: Sized,
{
    type Error;

    #[must_use]
    fn new(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
        initialization_args: InitializationArg,
    ) -> Result<Bundled<'a, Self>, Self::Error>;
    #[must_use]
    fn unpack(
        program_id: &'a Pubkey,
        accounts_iter: &mut std::slice::Iter<'a, AccountInfo<'a>>,
    ) -> Result<Bundled<'a, Self>, Self::Error>;
    #[must_use]
    fn pack(bundle: Bundled<'a, Self>) -> Result<(), Self::Error>;
}
