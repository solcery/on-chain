use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use super::Column as ColumnTrait;
use super::ColumnType;
use super::Error;
use solcery_impl_generator::generate_column_impls;

#[derive(
    PartialEq,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
/// Container for short-sized (16-byte) [`String`]
pub struct ShortString(String);

impl TryFrom<&str> for ShortString {
    type Error = TooLong;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let len = value.len();
        // 16 is a maximal total length, 4 - is a u32-encoded length of the string
        if len < 16 - 4 {
            Ok(Self(value.to_string()))
        } else {
            Err(TooLong {})
        }
    }
}

/// Container for medium-sized (64-byte) [`String`]
#[derive(
    PartialEq,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct MediumString(String);

impl TryFrom<&str> for MediumString {
    type Error = TooLong;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let len = value.len();
        // 64 is a maximal total length, 4 - is a u32-encoded length of the string
        if len < 64 - 4 {
            Ok(Self(value.to_string()))
        } else {
            Err(TooLong {})
        }
    }
}

/// Container for long (256-byte) [`String`]
#[derive(
    PartialEq,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct LongString(String);

impl TryFrom<&str> for LongString {
    type Error = TooLong;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let len = value.len();
        // 256 is a maximal total length, 4 - is a u32-encoded length of the string
        if len < 256 - 4 {
            Ok(Self(value.to_string()))
        } else {
            Err(TooLong {})
        }
    }
}

#[generate_column_impls(
    Data,
    ColumnTrait,
    Error,
    derives(
        PartialEq,
        Clone,
        Eq,
        Ord,
        PartialOrd,
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
    #[type_params(ShortString, 16)]
    ShortString,
    /// 64 bytes long [`String`]
    #[type_params(MediumString, 64)]
    MediumString,
    /// 256 bytes long [`String`]
    #[type_params(LongString, 256)]
    LongString,
}

/// This type will be thrown, if you try to convert a longer [`String`] to one of the
/// String-containers
#[derive(
    PartialEq,
    Copy,
    Clone,
    Eq,
    Ord,
    PartialOrd,
    Debug,
    BorshSerialize,
    BorshDeserialize,
    Serialize,
    Deserialize,
)]
pub struct TooLong;
