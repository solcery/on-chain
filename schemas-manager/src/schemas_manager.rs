use solcery_data_types::db::messages::schemas_manager::{
    AddSchema, GetSchema, RemoveSchema, UpdateSchema,
};

use crate::error::SchemasManagerError;
// use slice_rbtree::RBTree;

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
