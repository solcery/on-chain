use crate::db::schema_id::SchemaId;
use borsh::{BorshDeserialize, BorshSerialize};

// TODO: specify by DB
pub type DBId = String;
pub type DBQuery = String;

// TODO: replase on DB address
pub type DB = String;

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct CreateDB {
    pub schema_id: SchemaId,
    pub db_id: DBId,
    pub need_init: bool,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct RemoveDB {
    pub db_id: DBId,
}

#[derive(Debug, BorshSerialize, BorshDeserialize)]
pub struct Query {
    pub db_id: DBId,
    pub query: DBQuery,
}
