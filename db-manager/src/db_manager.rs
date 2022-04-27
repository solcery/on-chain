use solcery_data_types::db::messages::db_manager::{CreateDB, Query, RemoveDB};

use crate::error::DBManagerError;

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
