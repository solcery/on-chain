use borsh::{BorshDeserialize, BorshSerialize};
use generator::generate_column_impls;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

#[generate_column_impls]
pub enum DataType {
    #[type_params(i32, 4)]
    Int,
    #[type_params(Pubkey, 64)]
    Pubkey,
    #[type_params(String, 16)]
    ShortString,
    #[type_params(String, 64)]
    MediumString,
    #[type_params(String, 256)]
    LongString,
}

#[test]
fn panic() {
    panic!();
}
