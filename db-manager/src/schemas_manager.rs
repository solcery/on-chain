use borsh::{BorshDeserialize, BorshSerialize};
use slice_rbtree::RBTree;

//TODO: fill allowed types
enum AllowedTypes {}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub struct Schema {
    pub version: u64,
    tables: String, //TODO: how store tables order/types?
}

pub type SchemaId = String;

pub struct SchemasManager {
    // storage: RBTree,
}

impl SchemasManager {
    pub fn add_schema() {
        unimplemented!();
    }
    pub fn remove_schema() {
        unimplemented!();
    }
    pub fn update_schema() {
        unimplemented!();
    }
    pub fn get_schema(id: SchemaId) {
        unimplemented!();
    }
}
