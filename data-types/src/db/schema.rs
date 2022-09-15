use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

use super::column::Column as ColumnTrait;
use super::error::Error;
use solcery_impl_generator::generate_column_impls;

pub type Tables = Vec<AllowedTypes>;

#[must_use]
pub fn contains_one_primary_key(tables: &Tables) -> bool {
    tables.iter().fold(0, |acc, t| match *t {
        AllowedTypes::Int(KeyType::Primary)
        | AllowedTypes::String(KeyType::Primary)
        | AllowedTypes::Pubkey(KeyType::Primary) => acc + 1,
        _ => acc,
    }) == 1u64
}

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum KeyType {
    Primary,
    Secondary,
    NotKey,
    LongString(String), // 256 bytes
}

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

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum AllowedTypes {
    Int(KeyType),
    String(KeyType),
    Pubkey(KeyType),
}

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct ColumnParams {
    name: String,
    data_type: DataType,
    column_type: ColumnType,
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Schema {
    pub version: u64,
    pub tables: Tables,
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct NewSchema {
    pub version: u64,
    pub primary_key: DataType,
    pub colums: Vec<DataType>,
    pub max_colums: u32,
    pub max_rows: u32,
}

impl Schema {
    #[must_use]
    pub fn index_size(&self) -> usize {
        unimplemented!();
    }
}

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
pub enum ColumnType {
    RBTree,
    OneToOne,
    // This types are not implemented yet
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
