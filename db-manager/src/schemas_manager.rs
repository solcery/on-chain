use crate::schemas_error::SchemasManagerError;
use borsh::BorshSerialize;
use slice_rbtree::tree::RBTree;
use solcery_data_types::db::{
    messages::schemas_manager::{AddSchema, GetSchema, RemoveSchema, UpdateSchema},
    schema::{contains_one_primary_key, Schema},
    schema_id::SchemaId,
};
use std::cell::RefMut;

pub type SchemasHolderTree<'a> = RBTree<'a, SchemaId, Schema, 1024, 1024>;

#[derive(Debug)]
pub struct SchemasManager {}

impl SchemasManager {
    pub fn add_schema(
        message: AddSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        let mut schemas_holder = unsafe {
            if message.need_init {
                SchemasHolderTree::init_slice(data.as_mut()).unwrap()
            } else {
                SchemasHolderTree::from_slice(data.as_mut()).unwrap()
            }
        };

        if !contains_one_primary_key(&message.schema.tables) {
            return Err(SchemasManagerError::PrimaryKeyError);
        }

        schemas_holder.insert(message.id, message.schema).unwrap();

        Ok(())
    }

    pub fn remove_schema(
        message: RemoveSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        let mut schemas_holder = unsafe { SchemasHolderTree::from_slice(data.as_mut()).unwrap() };
        schemas_holder.delete(&message.id);

        Ok(())
    }

    pub fn update_schema(
        message: UpdateSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        let mut schemas_holder = unsafe { SchemasHolderTree::from_slice(data.as_mut()).unwrap() };

        if !contains_one_primary_key(&message.tables) {
            return Err(SchemasManagerError::PrimaryKeyError);
        }

        if schemas_holder.contains_key(&message.id) {
            let mut schema = schemas_holder.get(&message.id).unwrap();
            schema.version += 1;
            schema.tables = message.tables;

            schemas_holder.insert(message.id, schema).unwrap();
        }

        Ok(())
    }

    pub fn get_schema(
        message: GetSchema,
        mut data: RefMut<&mut [u8]>,
        mut res_data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        let schemas_holder = unsafe { SchemasHolderTree::from_slice(data.as_mut()).unwrap() };

        if let Some(schema) = schemas_holder.get(&message.id) {
            let mut v = vec![];
            schema.serialize(&mut v).unwrap();
            res_data.copy_from_slice(v.as_slice());
        };

        Ok(())
    }
}
