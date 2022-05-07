use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

//TODO: fill other allowed types

#[derive(PartialEq, Clone, Eq, Debug, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub enum AllowedTypes {
    Int,    // i64
    String, // String
}

#[derive(PartialEq, Eq, Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Schema {
    pub version: u64,
    pub tables: Vec<AllowedTypes>,
}
