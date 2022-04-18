mod error;
use self::error::DBManagerError;
use crate::schemas_manager::SchemaId;

// TODO: specify by DB
pub type DBId = String;
pub type DBQuery = String;

pub struct DBManager {}

impl DBManager {
    pub fn create_db(_schema_id: SchemaId, _db_id: DBId) -> Result<(), DBManagerError> {
        unimplemented!();
    }
    pub fn remove_db(_db_id: DBId) -> Result<(), DBManagerError> {
        unimplemented!();
    }
    pub fn process_query(_db_id: DBId, _query: DBQuery) -> Result<(), DBManagerError> {
        unimplemented!();
    }
}
