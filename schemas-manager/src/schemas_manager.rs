use crate::error::SchemasManagerError;
use borsh::BorshSerialize;
use slice_rbtree::RBTree;
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
        unsafe {
            let mut schemas_holder = match SchemasHolderTree::from_slice(data.as_mut()) {
                Ok(schemas_holder) => schemas_holder,
                Err(_) => SchemasHolderTree::init_slice(data.as_mut()).unwrap(),
            };
            schemas_holder.insert(message.id, message.schema).unwrap();
        }

        Ok(())
    }

    pub fn remove_schema(
        message: RemoveSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        unsafe {
            let mut schemas_holder = match SchemasHolderTree::from_slice(data.as_mut()) {
                Ok(schemas_holder) => schemas_holder,
                Err(_) => SchemasHolderTree::init_slice(data.as_mut()).unwrap(),
            };
            schemas_holder.delete(&message.id);
        }

        Ok(())
    }

    pub fn update_schema(
        message: UpdateSchema,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        unsafe {
            let mut schemas_holder = match SchemasHolderTree::from_slice(data.as_mut()) {
                Ok(schemas_holder) => schemas_holder,
                Err(_) => SchemasHolderTree::init_slice(data.as_mut()).unwrap(),
            };
            schemas_holder
                .insert(message.id, message.new_schema)
                .unwrap();
        }

        Ok(())
    }

    pub fn get_schema(
        message: GetSchema,
        mut data: RefMut<&mut [u8]>,
        mut res_data: RefMut<&mut [u8]>,
    ) -> Result<(), SchemasManagerError> {
        unsafe {
            let schemas_holder = match SchemasHolderTree::from_slice(data.as_mut()) {
                Ok(schemas_holder) => schemas_holder,
                Err(_) => SchemasHolderTree::init_slice(data.as_mut()).unwrap(),
            };

            match schemas_holder.get(&message.id) {
                Some(schema) => {
                    let mut v = vec![];
                    schema.serialize(&mut v).unwrap();
                    res_data.copy_from_slice(v.as_slice());
                }
                None => (),
            };
        }

        Ok(())
    }
}
