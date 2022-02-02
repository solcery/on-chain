use solana_program::account_info::AccountInfo;

use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Bundled<'a, T> {
    data: T,
    account: &'a AccountInfo<'a>,
}

impl<'a, T> Bundled<'a, T> {
    pub unsafe fn new(data: T, account: &'a AccountInfo<'a>) -> Self {
        Self { data, account }
    }

    pub unsafe fn release(self) -> (T, &'a AccountInfo<'a>) {
        (self.data, self.account)
    }

    #[must_use]
    pub fn key(&self) -> Pubkey {
        *self.account.key
    }

    #[must_use]
    pub fn data(&self) -> &T {
        &self.data
    }

    #[must_use]
    pub fn data_mut(&mut self) -> &mut T {
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
