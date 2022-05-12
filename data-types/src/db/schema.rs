use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

pub type Tables = Vec<AllowedTypes>;

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
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
