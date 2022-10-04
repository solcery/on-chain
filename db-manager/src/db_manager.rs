use std::cell::RefMut;

use slice_rbtree::tree::RBTree;
use solcery_data_types::db::messages::db_manager::{CreateDB, DBId, Query, RemoveDB, DB};

use crate::db_error::DBManagerError;

pub type DataBaseHolderTree<'a> = RBTree<'a, DBId, DB, 1024, 1024>;

#[derive(Debug)]
pub struct DBManager {}

impl DBManager {
    pub fn create_db(message: CreateDB, mut data: RefMut<&mut [u8]>) -> Result<(), DBManagerError> {
        let mut db_holder = unsafe {
            if message.need_init {
                DataBaseHolderTree::init_slice(data.as_mut()).unwrap()
            } else {
                DataBaseHolderTree::from_slice(data.as_mut()).unwrap()
            }
        };

        // TODO: db creation
        db_holder.insert(message.db_id, DB::default()).unwrap();
        // Obtain Schema
        // extract index_size
        // allocate index_size bytes
        // initialize DB in segment (DB::init_in_segment())
        // for each column: initialize columns (DB::add_column())

        Ok(())
    }

    pub fn remove_db(message: RemoveDB, mut data: RefMut<&mut [u8]>) -> Result<(), DBManagerError> {
        let mut db_holder = unsafe { DataBaseHolderTree::from_slice(data.as_mut()).unwrap() };
        db_holder.delete(&message.db_id);
        // Obtain DB from its SegmentId
        // DB::remove()

        Ok(())
    }

    pub fn process_query(
        message: Query,
        mut data: RefMut<&mut [u8]>,
        mut _res_data: RefMut<&mut [u8]>,
    ) -> Result<(), DBManagerError> {
        let mut _db_holder = unsafe { DataBaseHolderTree::from_slice(data.as_mut()).unwrap() };

        let _db = _db_holder.get(&message.db_id).unwrap();

        // TODO: process query
        unimplemented!();

        // Ok(())
    }
}
