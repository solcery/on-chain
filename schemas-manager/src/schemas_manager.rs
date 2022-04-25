use crate::{
    error::SchemasManagerError,
    messages::{AddSchema, GetSchema, RemoveSchema, UpdateSchema},
};
use borsh::{BorshDeserialize, BorshSerialize};
// use slice_rbtree::RBTree;

//TODO: fill other allowed types
#[derive(Debug, BorshSerialize, BorshDeserialize)]
enum AllowedTypes {
    Int,    // i64
    String, // String
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Schema {
    pub version: u64,
    tables: Vec<AllowedTypes>,
}

pub struct SchemasManager {}

#[allow(missing_docs)]
impl SchemasManager {
    pub fn add_schema(_message: AddSchema) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn remove_schema(_message: RemoveSchema) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn update_schema(_message: UpdateSchema) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn get_schema(_message: GetSchema) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
}
