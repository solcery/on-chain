use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub type Tables = Vec<AllowedTypes>;

pub fn contains_one_primary_key(tables: &Tables) -> bool {
    tables.iter().fold(0, |acc, t| match *t {
        AllowedTypes::Int(KeyType::Primary)
        | AllowedTypes::String(KeyType::Primary)
        | AllowedTypes::Pubkey(KeyType::Primary) => acc + 1,
        _ => acc,
    }) == 1u64
}

#[derive(
    PartialEq, Copy, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
pub enum KeyType {
    Primary,
    Secondary,
    NotKey,
}

//TODO: fill other allowed types

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum AllowedTypes {
    Int(KeyType),
    String(KeyType),
    Pubkey(KeyType),
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Schema {
    pub version: u64,
    pub tables: Tables,
}
