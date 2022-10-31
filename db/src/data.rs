use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use super::Column as ColumnTrait;
use super::ColumnType;
use super::Error;
use solcery_impl_generator::generate_column_impls;

#[generate_column_impls(
    Data,
    ColumnTrait,
    Error,
    derives(
        PartialEq,
        Clone,
        Eq,
        Debug,
        BorshSerialize,
        BorshDeserialize,
        Serialize,
        Deserialize
    )
)]
#[derive(
    PartialEq,
    Copy,
    Clone,
    Eq,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
    TryFromPrimitive,
    IntoPrimitive,
)]
#[repr(u8)]
/// Data types, that can be stored in the [`DB`](crate::DB)
pub enum DataType {
    /// [`i32`]
    #[type_params(i32, 4)]
    Int,
    /// [`Pubkey`]
    #[type_params(Pubkey, 64)]
    Pubkey,
    /// 16 bytes long [`String`]
    #[type_params(String, 16)]
    ShortString,
    /// 64 bytes long [`String`]
    #[type_params(String, 64)]
    MediumString,
    /// 256 bytes long [`String`]
    #[type_params(String, 256)]
    LongString,
}
