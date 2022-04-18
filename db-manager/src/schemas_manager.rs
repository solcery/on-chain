use self::error::SchemasManagerError;
use borsh::{BorshDeserialize, BorshSerialize};
// use slice_rbtree::RBTree;

mod error;

//TODO: fill allowed types
enum _AllowedTypes {}

#[allow(missing_docs)]
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Debug, BorshSerialize, BorshDeserialize)]
pub struct Schema {
    pub version: u64,
    tables: String, // TODO: how store tables order/types?
}

pub type SchemaId = String;
pub struct SchemasManager {
    // storage: RBTree,
}

#[allow(missing_docs)]
impl SchemasManager {
    pub fn add_schema(_schema_id: SchemaId, _schema: Schema) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn remove_schema(_id: SchemaId) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn update_schema(
        _schema_id: SchemaId,
        _new_schema: Schema,
    ) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
    pub fn get_schema(_id: SchemaId) -> Result<(), SchemasManagerError> {
        unimplemented!();
    }
}
