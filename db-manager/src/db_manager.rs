use std::cell::RefMut;

use slice_rbtree::RBTree;
use solcery_data_types::db::messages::db_manager::{CreateDB, DBId, Query, RemoveDB, DB};

use crate::db_error::DBManagerError;

pub type DataBaseHolderTree<'a> = RBTree<'a, DBId, DB, 1024, 1024>;
pub struct DBManager {}

impl DBManager {
    pub fn create_db(message: CreateDB, mut data: RefMut<&mut [u8]>) -> Result<(), DBManagerError> {
        unsafe {
            let mut db_holder = match DataBaseHolderTree::from_slice(data.as_mut()) {
                Ok(db_holder) => db_holder,
                Err(_) => DataBaseHolderTree::init_slice(data.as_mut()).unwrap(),
            };

            // TODO: db creation
            db_holder.insert(message.db_id, DB::default()).unwrap();
        }

        Ok(())
    }

    pub fn remove_db(message: RemoveDB, mut data: RefMut<&mut [u8]>) -> Result<(), DBManagerError> {
        unsafe {
            let mut db_holder = match DataBaseHolderTree::from_slice(data.as_mut()) {
                Ok(db_holder) => db_holder,
                Err(_) => DataBaseHolderTree::init_slice(data.as_mut()).unwrap(),
            };
            db_holder.delete(&message.db_id);
        }

        Ok(())
    }

    pub fn process_query(
        message: Query,
        mut data: RefMut<&mut [u8]>,
    ) -> Result<(), DBManagerError> {
        unsafe {
            let db_holder = match DataBaseHolderTree::from_slice(data.as_mut()) {
                Ok(db_holder) => db_holder,
                Err(_) => DataBaseHolderTree::init_slice(data.as_mut()).unwrap(),
            };

            let _db = db_holder.get(&message.db_id).unwrap();

            // TODO: process query
            unimplemented!();
        }

        Ok(())
    }
}
