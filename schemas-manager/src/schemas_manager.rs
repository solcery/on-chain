use crate::error::SchemasManagerError;
use slice_rbtree::RBTree;
use solana_program::msg;
use solcery_data_types::db::{
    messages::schemas_manager::{AddSchema, GetSchema, RemoveSchema, UpdateSchema},
    schema::Schema,
    schema_id::SchemaId,
};
use std::cell::RefMut;

pub type SchemasHolderTree<'a> = RBTree<'a, SchemaId, Schema, 1024, 1024>;

pub struct SchemasManager {}

#[allow(missing_docs)]
impl SchemasManager {
    pub fn add_schema(
        message: AddSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        msg!("data: {:?}", data);

        dbg!(SchemasHolderTree::expected_size(1));

        let mut schemas_holder = SchemasHolderTree::init_slice(data.as_mut()).unwrap();

        let res = schemas_holder.insert(message.id, message.schema);

        msg!("insert res: {:?}", res);

        Ok(())
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
