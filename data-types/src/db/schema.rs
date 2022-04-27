use borsh::{BorshDeserialize, BorshSerialize};

//TODO: fill other allowed types
#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub enum AllowedTypes {
    Int,    // i64
    String, // String
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Schema {
    pub version: u64,
    pub tables: Vec<AllowedTypes>,
}
