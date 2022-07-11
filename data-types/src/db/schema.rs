use borsh::{BorshDeserialize, BorshSerialize};
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use serde::{Deserialize, Serialize};
use solana_program::pubkey::Pubkey;

pub type Tables = Vec<AllowedTypes>;

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
    Int,
    Pubkey,
    ShortString,
    MediumString,
    LongString,
}

impl DataType {
    pub const fn size(&self) -> usize {
        use DataType::*;
        match self {
            Int => 4,
            Pubkey => 64,
            ShortString => 16,
            MediumString => 64,
            LongString => 256,
        }
    }
}

#[derive(PartialEq, Clone, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum Data {
    Int(i64),
    Pubkey(Pubkey),
    ShortString(String),  // 16 bytes
    MediumString(String), // 64 bytes
    LongString(String),   // 256 bytes
}

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum AllowedTypes {
    Int(KeyType),
    String(KeyType),
    Pubkey(KeyType),
}

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Column {
    name: String,
    typ: DataType,
    is_secondary_key: bool,
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
    // This types are not implemented yet
    //OneToOne,
    //OneToMany,
    //ManyToOne,
    //ManyToMany,
    //RBSet,
}
