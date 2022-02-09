use solana_program::account_info::AccountInfo;

use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub struct Bundled<'s, 't0, T> {
    data: T,
    account: &'s AccountInfo<'t0>,
}

impl<'s, 't0, T> Bundled<'s, 't0, T> {
    pub unsafe fn new(data: T, account: &'s AccountInfo<'t0>) -> Self {
        Self { data, account }
    }

    pub unsafe fn release(self) -> (T, &'s AccountInfo<'t0>) {
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

impl<'s, 't0, T: PartialEq> PartialEq for Bundled<'s, 't0, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

pub trait Bundle<'r, 's, 't0, 't1, InitializationArg>
where
    Self: Sized,
{
    type Error;

    fn new(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
        initialization_args: InitializationArg,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>;

    fn unpack(
        program_id: &'r Pubkey,
        accounts_iter: &mut std::slice::Iter<'s, AccountInfo<'t0>>,
    ) -> Result<Bundled<'s, 't0, Self>, Self::Error>;

    fn pack(bundle: Bundled<'s, 't0, Self>) -> Result<(), Self::Error>;
}
