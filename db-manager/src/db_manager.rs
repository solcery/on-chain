use crate::{
    error::DBManagerError,
    messages::{CreateDB, Query, RemoveDB},
};

// TODO: specify by DB
pub type DBId = String;
pub type DBQuery = String;

pub struct DBManager {}

impl DBManager {
    pub fn create_db(_message: CreateDB) -> Result<(), DBManagerError> {
        unimplemented!();
    }
    pub fn remove_db(_message: RemoveDB) -> Result<(), DBManagerError> {
        unimplemented!();
    }
    pub fn process_query(_message: Query) -> Result<(), DBManagerError> {
        unimplemented!();
    }
}
